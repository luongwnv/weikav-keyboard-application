//! Nonexclusive, read-only NUT75 Consumer Control input. Active only in the focused tester.
#[cfg(target_os = "macos")]
mod macos;
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

#[derive(Clone, Debug)]
struct Field {
    report: u8,
    offset: usize,
    bits: usize,
    count: usize,
    variable: bool,
    usages: Vec<u32>,
    minimum: u32,
}
#[derive(Clone, Default)]
struct Global {
    page: u32,
    size: usize,
    count: usize,
    report: u8,
}

fn fields(descriptor: &[u8]) -> Vec<Field> {
    let mut global = Global::default();
    let mut stack = Vec::new();
    let mut offsets = BTreeMap::<u8, usize>::new();
    let (mut usages, mut minimum) = (Vec::new(), 0);
    let mut result = Vec::new();
    let mut at = 0;
    while at < descriptor.len() {
        let prefix = descriptor[at];
        at += 1;
        if prefix == 0xfe {
            if at + 2 > descriptor.len() {
                break;
            }
            let n = descriptor[at] as usize;
            at += 2 + n;
            continue;
        }
        let len = match prefix & 3 {
            3 => 4,
            n => n as usize,
        };
        if at + len > descriptor.len() {
            break;
        }
        let value = descriptor[at..at + len]
            .iter()
            .enumerate()
            .fold(0u32, |v, (i, b)| v | u32::from(*b) << (i * 8));
        at += len;
        let kind = (prefix >> 2) & 3;
        let tag = prefix >> 4;
        match (kind, tag) {
            (1, 0) => global.page = value,
            (1, 7) => global.size = value as usize,
            (1, 8) => global.report = value as u8,
            (1, 9) => global.count = value as usize,
            (1, 10) => stack.push(global.clone()),
            (1, 11) => {
                if let Some(g) = stack.pop() {
                    global = g
                }
            }
            (2, 0) => usages.push(value),
            (2, 1) => minimum = value,
            (0, 8) => {
                let offset = *offsets.entry(global.report).or_default();
                if global.page == 12
                    && value & 1 == 0
                    && (1..=32).contains(&global.size)
                    && global.count <= 256
                {
                    result.push(Field {
                        report: global.report,
                        offset,
                        bits: global.size,
                        count: global.count,
                        variable: value & 2 != 0,
                        usages: usages.clone(),
                        minimum,
                    });
                }
                *offsets.entry(global.report).or_default() =
                    offset.saturating_add(global.size.saturating_mul(global.count));
            }
            _ => {}
        }
        if kind == 0 {
            usages.clear();
            minimum = 0;
        }
    }
    result.retain(|f| {
        !f.variable
            || f.usages.iter().any(|u| [0xe2, 0xe9, 0xea].contains(u))
            || ([0xe2, 0xe9, 0xea]
                .iter()
                .any(|u| *u >= f.minimum && *u < f.minimum + f.count as u32)
                && f.usages.is_empty())
    });
    result
}
fn decode(fields: &[Field], report: &[u8]) -> Vec<u8> {
    let mut pulses = Vec::new();
    for field in fields {
        let data = if field.report == 0 {
            report
        } else {
            if report.first() != Some(&field.report) {
                continue;
            }
            &report[1..]
        };
        for index in 0..field.count {
            let offset = field.offset + index * field.bits;
            if offset + field.bits > data.len() * 8 {
                break;
            }
            let value = (0..field.bits).fold(0u32, |value, bit| {
                value | u32::from((data[(offset + bit) / 8] >> ((offset + bit) % 8)) & 1) << bit
            });
            let usage = if field.variable {
                if value == 0 {
                    continue;
                }
                field
                    .usages
                    .get(index)
                    .copied()
                    .unwrap_or(field.minimum + index as u32)
            } else {
                value
            };
            let slot = match usage {
                0xe9 => 13,
                0xea => 14,
                0xe2 => 15,
                _ => continue,
            };
            if !pulses.contains(&slot) {
                pulses.push(slot);
            }
        }
    }
    pulses
}
struct Input {
    device: hidapi::HidDevice,
    fields: Vec<Field>,
}
fn input_paths(api: &hidapi::HidApi) -> HashSet<Vec<u8>> {
    api.device_list()
        .filter(|d| {
            d.vendor_id() == crate::device::VENDOR
                && [crate::device::PRODUCT_USB, crate::device::PRODUCT_RECEIVER]
                    .contains(&d.product_id())
                && d.product_string()
                    .is_some_and(|n| n.to_ascii_uppercase().contains("NUT75"))
                && d.usage_page() < 0xff00
        })
        .map(|d| d.path().to_bytes().to_vec())
        .collect()
}
fn open_inputs() -> Result<(hidapi::HidApi, Vec<Input>)> {
    let api = hidapi::HidApi::new()?;
    let mut inputs = Vec::new();
    let mut last_error = None;
    let mut paths = HashSet::new();
    for info in api.device_list().filter(|d| {
        d.vendor_id() == crate::device::VENDOR
            && [crate::device::PRODUCT_USB, crate::device::PRODUCT_RECEIVER]
                .contains(&d.product_id())
            && d.product_string()
                .is_some_and(|n| n.to_ascii_uppercase().contains("NUT75"))
            && d.usage_page() < 0xff00
    }) {
        if !paths.insert(info.path().to_bytes().to_vec()) {
            continue;
        }
        match (|| -> Result<Option<Input>> {
            let device = api.open_path(info.path())?;
            let mut descriptor = [0u8; 4096];
            let size = device.get_report_descriptor(&mut descriptor)?;
            let fields = fields(&descriptor[..size]);
            Ok(if fields.is_empty() {
                None
            } else {
                Some(Input { device, fields })
            })
        })() {
            Ok(Some(input)) => inputs.push(input),
            Ok(None) => {}
            Err(e) => last_error = Some(e),
        }
    }
    if inputs.is_empty() {
        return Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Chưa thấy giao diện núm NUT75")));
    }
    Ok((api, inputs))
}

pub fn caps_lock() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::caps_lock()
    }
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetKeyState(key: i32) -> i16;
        }
        unsafe { GetKeyState(0x14) & 1 != 0 }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}
#[cfg(target_os = "windows")]
pub fn modifier_states() -> [(u16, bool); 8] {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetKeyState(key: i32) -> i16;
    }
    [
        (224, 0xa2),
        (225, 0xa0),
        (226, 0xa4),
        (227, 0x5b),
        (228, 0xa3),
        (229, 0xa1),
        (230, 0xa5),
        (231, 0x5c),
    ]
    .map(|(usage, key)| (usage, unsafe { GetKeyState(key) < 0 }))
}

pub enum InputEvent {
    Pulse(u8),
    #[cfg(target_os = "macos")]
    Key {
        usage: u16,
        pressed: bool,
    },
    Ready,
    Unavailable(String),
}
pub struct Monitor {
    active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    pub events: mpsc::Receiver<InputEvent>,
    #[cfg(target_os = "macos")]
    _local: Option<macos::LocalMonitor>,
}
impl Monitor {
    pub fn start(ctx: eframe::egui::Context) -> Self {
        let active = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, events) = mpsc::channel();
        #[cfg(target_os = "macos")]
        let local = macos::LocalMonitor::start(active.clone(), sender.clone(), ctx.clone());
        #[cfg(target_os = "macos")]
        let use_hid = local.is_none();
        #[cfg(not(target_os = "macos"))]
        let use_hid = true;
        let worker_active = active.clone();
        let worker_stop = stop.clone();
        if use_hid {
            thread::Builder::new()
                .name("nut75-consumer-input".into())
                .spawn(move || {
                    let mut capture = None;
                    let mut retry = std::time::Instant::now();
                    let mut reported = false;
                    let mut captured_paths = HashSet::new();
                    let mut last_scan = std::time::Instant::now();
                    while !worker_stop.load(Ordering::Relaxed) {
                        if !worker_active.load(Ordering::Relaxed) {
                            capture = None;
                            reported = false;
                            retry = std::time::Instant::now();
                            thread::sleep(Duration::from_millis(40));
                            continue;
                        }
                        if capture.is_none() && retry.elapsed() >= Duration::from_millis(100) {
                            match open_inputs() {
                                Ok(input) => {
                                    captured_paths = input_paths(&input.0);
                                    last_scan = std::time::Instant::now();
                                    capture = Some(input);
                                    let _ = sender.send(InputEvent::Ready);
                                    ctx.request_repaint();
                                    reported = false;
                                }
                                Err(e) => {
                                    if !reported {
                                        let _ =
                                            sender.send(InputEvent::Unavailable(format!("{e:#}")));
                                        ctx.request_repaint();
                                        reported = true;
                                    }
                                    retry = std::time::Instant::now() + Duration::from_secs(1);
                                }
                            }
                        }
                        if last_scan.elapsed() >= Duration::from_secs(1) {
                            let changed = if let Some((api, _)) = &mut capture {
                                api.refresh_devices().is_err() || input_paths(api) != captured_paths
                            } else {
                                false
                            };
                            if changed {
                                capture = None;
                                retry = std::time::Instant::now();
                            }
                            last_scan = std::time::Instant::now();
                        }
                        let mut failed = false;
                        if let Some((_, inputs)) = &capture {
                            for input in inputs {
                                for _ in 0..32 {
                                    let mut report = [0; 1024];
                                    match input.device.read_timeout(&mut report, 0) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            for slot in decode(&input.fields, &report[..n]) {
                                                let _ = sender.send(InputEvent::Pulse(slot));
                                                ctx.request_repaint();
                                            }
                                        }
                                        Err(_) => {
                                            failed = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if failed {
                            capture = None;
                            retry = std::time::Instant::now();
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                })
                .expect("start input monitor");
        }
        Self {
            active,
            stop,
            events,
            #[cfg(target_os = "macos")]
            _local: local,
        }
    }
    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
pub fn probe() -> Result<()> {
    let (_api, inputs) = open_inputs().context("Không mở được Consumer Control NUT75")?;
    for input in inputs {
        println!("Consumer fields: {:?}", input.fields);
    }
    Ok(())
}
/// Read-only inventory of NUT75 input/report descriptors, including Bluetooth.
pub fn probe_bluetooth() -> Result<()> {
    let api = hidapi::HidApi::new()?;
    let mut paths = HashSet::new();
    for info in api.device_list().filter(|d| {
        d.product_string()
            .is_some_and(|s| s.to_ascii_uppercase().contains("NUT75"))
    }) {
        println!(
            "{} | {:04X}/{:04X} | bus {:?} | page {:04X}, usage {:04X}",
            info.product_string().unwrap_or("NUT75"),
            info.vendor_id(),
            info.product_id(),
            info.bus_type(),
            info.usage_page(),
            info.usage()
        );
        if paths.insert(info.path().to_bytes().to_vec()) {
            match api.open_path(info.path()) {
                Ok(device) => {
                    let mut bytes = [0; 4096];
                    let n = device.get_report_descriptor(&mut bytes)?;
                    println!("Descriptor: {:02X?}", &bytes[..n]);
                }
                Err(e) => println!("Descriptor unavailable: {e}"),
            }
        }
    }
    Ok(())
}
/// Short focused diagnostic: prints only Consumer Control reports, never key reports.
pub fn watch() -> Result<()> {
    let (_api, inputs) = open_inputs()?;
    println!("Watching Consumer Control for 20 seconds; rotate and press the NUT75 wheel.");
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        for input in &inputs {
            let mut raw = [0; 1024];
            let n = input.device.read_timeout(&mut raw, 0)?;
            if n > 0
                && input
                    .fields
                    .iter()
                    .any(|f| f.report == 0 || raw[0] == f.report)
            {
                println!(
                    "Consumer: {:02X?} => {:?}",
                    &raw[..n],
                    decode(&input.fields, &raw[..n])
                );
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consumer_arrays_and_bitfields_respect_report_ids_and_offsets() {
        let array = fields(&[
            0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0x85, 3, 0x75, 16, 0x95, 1, 0x19, 0, 0x2a, 0xff, 3,
            0x81, 0, 0xc0,
        ]);
        assert_eq!(decode(&array, &[3, 0xe9, 0]), vec![13]);
        assert_eq!(decode(&array, &[3, 0xea, 0]), vec![14]);
        assert_eq!(decode(&array, &[3, 0xe2, 0]), vec![15]);
        assert!(decode(&array, &[4, 0xe9, 0]).is_empty());
        assert!(decode(&array, &[3, 0, 0]).is_empty());
        let bits = fields(&[
            0x05, 0x0c, 0x85, 2, 0x75, 3, 0x95, 1, 0x81, 1, 0x75, 1, 0x95, 3, 0x09, 0xe2, 0x09,
            0xe9, 0x09, 0xea, 0x81, 2,
        ]);
        assert_eq!(decode(&bits, &[2, 0b00010000]), vec![13]);
        assert_eq!(decode(&bits, &[2, 0b00001000]), vec![15]);
    }
}
