//! Observe only media events dispatched to this app. No global event tap or key suppression.
use super::InputEvent;
use block2::RcBlock;
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};
use std::{
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

pub struct LocalMonitor(Retained<AnyObject>);
impl LocalMonitor {
    pub fn start(
        active: Arc<AtomicBool>,
        sender: mpsc::Sender<InputEvent>,
        ctx: eframe::egui::Context,
    ) -> Option<Self> {
        let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
            // AppKit invokes local monitors on the UI thread. Return the borrowed event
            // unchanged so system volume/mute behavior continues normally.
            let e = unsafe { event.as_ref() };
            if active.load(Ordering::Relaxed) && e.r#type() == NSEventType::FlagsChanged {
                if let Some((usage, pressed)) =
                    modifier_key(e.keyCode(), e.modifierFlags().0 as u64)
                {
                    let _ = sender.send(InputEvent::Key { usage, pressed });
                }
                ctx.request_repaint();
            }
            if active.load(Ordering::Relaxed)
                && e.r#type() == NSEventType::SystemDefined
                && let Some(slot) = media_slot(e.subtype().0, e.data1() as u32)
            {
                let _ = sender.send(InputEvent::Pulse(slot));
                ctx.request_repaint();
            }
            event.as_ptr()
        });
        // Installed during App creation on the main thread; AppKit copies the block.
        unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                NSEventMask::SystemDefined | NSEventMask::FlagsChanged,
                &handler,
            )
        }
        .map(Self)
    }
}
impl Drop for LocalMonitor {
    fn drop(&mut self) {
        unsafe { NSEvent::removeMonitor(&self.0) };
    }
}

// Values from the macOS SDK IOLLEvent.h and ev_keymap.h. Ignore releases and
// unrelated system-defined events; leave the original AppKit event untouched.
fn media_slot(subtype: i16, data: u32) -> Option<u8> {
    if subtype != 8 || (data >> 8) & 0xff != 0x0a {
        return None;
    }
    match data >> 16 {
        0 => Some(13),
        1 => Some(14),
        7 => Some(15),
        _ => None,
    }
}
// Carbon virtual key codes and IOLLEvent.h device-specific flags preserve left/right.
// Fn is reported only if the keyboard exposes it to macOS (many firmwares keep it internal).
fn modifier_key(code: u16, flags: u64) -> Option<(u16, bool)> {
    let (usage, mask) = match code {
        59 => (224, 0x0001),
        56 => (225, 0x0002),
        58 => (226, 0x0020),
        55 => (227, 0x0008),
        62 => (228, 0x2000),
        60 => (229, 0x0004),
        61 => (230, 0x0040),
        54 => (231, 0x0010),
        63 => (175, 0x0080_0000),
        _ => return None,
    };
    Some((usage, flags & mask != 0))
}
pub fn caps_lock() -> bool {
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::CapsLock)
}

#[cfg(test)]
mod tests {
    use super::{media_slot, modifier_key};
    #[test]
    fn modifier_events_distinguish_sides_and_release_with_other_side_held() {
        assert_eq!(modifier_key(56, 0x20006), Some((225, true)));
        assert_eq!(modifier_key(60, 0x20006), Some((229, true)));
        assert_eq!(modifier_key(56, 0x20004), Some((225, false)));
        for (code, usage, flag) in [
            (59, 224, 1),
            (58, 226, 0x20),
            (55, 227, 8),
            (62, 228, 0x2000),
            (61, 230, 0x40),
            (54, 231, 0x10),
            (63, 175, 0x800000),
        ] {
            assert_eq!(modifier_key(code, flag), Some((usage, true)));
            assert_eq!(modifier_key(code, 0), Some((usage, false)));
        }
        assert_eq!(modifier_key(0, 0x20000), None);
    }
    #[test]
    fn appkit_media_reports_accept_presses_and_repeats_but_not_releases() {
        assert_eq!(media_slot(8, 0x00000a00), Some(13));
        assert_eq!(media_slot(8, 0x00010a01), Some(14));
        assert_eq!(media_slot(8, 0x00070a00), Some(15));
        assert_eq!(media_slot(8, 0x00070b00), None);
        assert_eq!(media_slot(7, 0x00070a00), None);
        assert_eq!(media_slot(8, 0x00100a00), None);
    }
}
