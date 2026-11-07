//! Native data formats from the NUT75 driver. See docs/protocol.md.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroAction {
    pub delay: u16,
    pub key: u8,
    /// Firmware action type, including the press bit. Preserve unknown action types on read.
    pub status: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<MacroAction>,
}

impl Macro {
    /// Move one complete action (including its delay) to an insertion boundary.
    pub fn move_action(&mut self, from: usize, insertion: usize) -> Option<usize> {
        if from >= self.actions.len() || insertion > self.actions.len() {
            return None;
        }
        let to = insertion - usize::from(from < insertion);
        if to != from {
            let action = self.actions.remove(from);
            self.actions.insert(to, action);
        }
        Some(to)
    }
    pub fn release_held(&mut self) {
        let mut held = BTreeMap::new();
        for action in &self.actions {
            let identity = (action.key, action.status & 0x70);
            if action.status & 0x80 != 0 {
                held.insert(identity, action.status & 0x7f);
            } else {
                held.remove(&identity);
            }
        }
        for ((key, _), status) in held {
            self.actions.push(MacroAction {
                delay: 0,
                key,
                status,
            });
        }
    }
}

pub fn macro_bytes(macros: &BTreeMap<u8, Macro>) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut table = vec![0; 400];
    let mut body = Vec::new();
    for (&id, m) in macros {
        ensure!(id < 100, "Số macro phải từ 1–100");
        ensure!(m.actions.len() <= 127, "Macro có quá nhiều thao tác");
        if m.actions.is_empty() {
            continue;
        }
        let address = (400 + body.len()) as u32;
        table[id as usize * 4..id as usize * 4 + 4].copy_from_slice(&address.to_le_bytes());
        body.extend_from_slice(&((m.actions.len() * 2) as u16).to_le_bytes());
        body.extend_from_slice(&[0, 0]);
        for action in &m.actions {
            body.extend_from_slice(&action.delay.to_le_bytes());
            body.extend_from_slice(&[action.key, action.status]);
        }
    }
    // NUT75's official macroConfig uses 512 bytes for blocks, excluding its 400-byte index.
    ensure!(body.len() <= 512, "Bộ nhớ macro NUT75 vượt quá 512 byte");
    Ok((table, body))
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardSettings {
    pub game_mode: u8,
    pub fn_switch: u8,
    pub sleep: u8,
    pub debounce: u8,
    pub report_rate: u8,
    pub system: u8,
    pub nkro: u8,
}
impl KeyboardSettings {
    pub fn from_raw(raw: &[u8]) -> Self {
        Self {
            game_mode: raw[1],
            fn_switch: raw[2],
            sleep: raw[3],
            debounce: raw[4],
            report_rate: raw[5],
            system: raw[6],
            nkro: raw[17],
        }
    }
    pub fn merge_into(&self, raw: &mut [u8]) {
        for (id, value) in [
            (1, self.game_mode),
            (2, self.fn_switch),
            (3, self.sleep),
            (4, self.debounce),
            (5, self.report_rate),
            (6, self.system),
            (17, self.nkro),
        ] {
            raw[id] = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ending_recording_releases_held_keys_and_mouse_without_duplicate_releases() {
        let mut m = Macro {
            name: "Test".into(),
            actions: vec![
                MacroAction {
                    delay: 0,
                    key: 224,
                    status: 0x90,
                },
                MacroAction {
                    delay: 5,
                    key: 6,
                    status: 0x90,
                },
                MacroAction {
                    delay: 5,
                    key: 6,
                    status: 0x10,
                },
                MacroAction {
                    delay: 5,
                    key: 1,
                    status: 0xb0,
                },
            ],
        };
        m.release_held();
        assert_eq!(m.actions.len(), 6);
        assert!(m.actions.iter().any(|a| a.key == 224 && a.status == 0x10));
        assert!(m.actions.iter().any(|a| a.key == 1 && a.status == 0x30));
        m.release_held();
        assert_eq!(m.actions.len(), 6);
    }
    #[test]
    fn macro_index_points_to_blocks_and_enforces_real_memory_limit() {
        let m = Macro {
            name: "Copy".into(),
            actions: vec![
                MacroAction {
                    delay: 300,
                    key: 6,
                    status: 0x90,
                },
                MacroAction {
                    delay: 50,
                    key: 6,
                    status: 0x10,
                },
            ],
        };
        let (table, body) = macro_bytes(&BTreeMap::from([(3, m.clone())])).unwrap();
        assert_eq!(&table[12..16], &400u32.to_le_bytes());
        assert_eq!(&body, &[4, 0, 0, 0, 44, 1, 6, 0x90, 50, 0, 6, 0x10]);
        let mut large = m;
        large.actions = vec![large.actions[0].clone(); 127];
        assert!(macro_bytes(&BTreeMap::from([(0, large.clone())])).is_ok());
        assert!(macro_bytes(&BTreeMap::from([(0, large.clone()), (1, large)])).is_err());
    }
    #[test]
    fn settings_change_preserves_unexposed_firmware_fields() {
        let mut raw = vec![7; 56];
        let mut settings = KeyboardSettings::from_raw(&raw);
        settings.debounce = 3;
        settings.merge_into(&mut raw);
        assert_eq!(raw[4], 3);
        assert_eq!(raw[18], 7);
        assert_eq!(raw[55], 7);
    }
}

#[cfg(test)]
mod reorder_tests {
    use super::*;
    #[test]
    fn moving_actions_preserves_delay_and_order_in_both_directions() {
        let mut m = Macro {
            name: "Test".into(),
            actions: (4..8)
                .map(|key| MacroAction {
                    key,
                    delay: u16::from(key) * 10,
                    status: 0x90,
                })
                .collect(),
        };
        assert_eq!(m.move_action(0, 4), Some(3));
        assert_eq!(
            m.actions.iter().map(|a| a.key).collect::<Vec<_>>(),
            vec![5, 6, 7, 4]
        );
        assert_eq!(m.actions[3].delay, 40);
        assert_eq!(m.move_action(3, 0), Some(0));
        let before = m.clone();
        assert_eq!(m.move_action(1, 2), Some(1));
        assert_eq!(m, before);
        assert_eq!(m.move_action(4, 0), None);
        assert_eq!(m.move_action(0, 5), None);
        assert_eq!(m, before);
    }
}
