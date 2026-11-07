use std::{collections::BTreeMap, ffi::CString, sync::mpsc, thread, time::Duration};

use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::{
    model::{self, Binding, Lighting, Profile, SideLighting},
    protocol::{self, Protocol},
};

pub const VENDOR: u16 = 0x0c45;
pub const PRODUCT_USB: u16 = 0x880c;
// The official driver lists decimal 34828 = 0x880C.
pub const PRODUCT_RECEIVER: u16 = 0xfef9;
const CONFIG_PAGES: &[u16] = &[65384, 65408, 65376, 65280, 65281, 65307];

#[derive(Clone, Debug, Serialize)]
pub struct DeviceEntry {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub supported: bool,
    pub path: String,
    #[serde(skip)]
    path_bytes: Vec<u8>,
}

fn is_nut75(name: &str) -> bool {
    name.to_ascii_uppercase()
        .replace([' ', '-', '_'], "")
        .contains("NUT75")
}

pub fn enumerate() -> Result<Vec<DeviceEntry>> {
    let api = hidapi::HidApi::new().context("Không khởi tạo được HID")?;
    Ok(api
        .device_list()
        .filter(|info| info.vendor_id() == VENDOR && CONFIG_PAGES.contains(&info.usage_page()))
        .map(|info| {
            let name = info.product_string().unwrap_or("Thiết bị HID").to_owned();
            DeviceEntry {
                supported: is_nut75(&name)
                    && [PRODUCT_USB, PRODUCT_RECEIVER].contains(&info.product_id()),
                name,
                vendor_id: info.vendor_id(),
                product_id: info.product_id(),
                usage_page: info.usage_page(),
                usage: info.usage(),
                interface: info.interface_number(),
                path: info.path().to_string_lossy().into_owned(),
                path_bytes: info.path().to_bytes().to_vec(),
            }
        })
        .collect())
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub version: String,
    pub battery: u8,
    pub charging: bool,
    pub work_mode: u8,
    pub profile: u8,
    pub frame_version: u8,
    pub manufacturer: u16,
    pub product: u16,
}

impl DeviceInfo {
    fn parse(raw: &[u8]) -> Result<Self> {
        ensure!(raw.len() == 48, "Thông tin thiết bị cần có 48 byte");
        ensure!(
            u16::from_le_bytes([raw[4], raw[5]]) == VENDOR,
            "Thiết bị trả về VID khác WEIKAV"
        );
        ensure!(
            [PRODUCT_USB, PRODUCT_RECEIVER].contains(&u16::from_le_bytes([raw[6], raw[7]])),
            "Thiết bị trả về PID khác NUT75"
        );
        ensure!(
            raw[32] == 0,
            "Firmware đang ở trạng thái phục hồi; không chỉnh cấu hình"
        );
        let minor = (raw[8] & 15) + ((raw[8] >> 4) * 10);
        Ok(Self {
            version: format!("{}.{minor:02}", raw[9]),
            battery: raw[17],
            charging: raw[18] != 0,
            work_mode: raw[16],
            profile: raw[19],
            frame_version: raw[30],
            manufacturer: u16::from_le_bytes([raw[12], raw[13]]),
            product: u16::from_le_bytes([raw[14], raw[15]]),
        })
    }
}

#[derive(Clone)]
pub struct Snapshot {
    pub entry: DeviceEntry,
    pub info: DeviceInfo,
    pub base: Vec<u8>,
    pub function: Vec<u8>,
    pub light: Vec<u8>,
    pub side: Option<Vec<u8>>,
    pub report_size: usize,
    pub custom: Vec<u8>,
    pub settings: Vec<u8>,
    pub macros: BTreeMap<u8, crate::features::Macro>,
}

impl Snapshot {
    pub fn binding(&self, function: bool, key: u8) -> Binding {
        let raw = if function { &self.function } else { &self.base };
        let offset = usize::from(key) * 4;
        raw[offset..offset + 4].try_into().unwrap()
    }

    pub fn lighting(&self) -> Lighting {
        Lighting::from_raw(&self.light)
    }

    pub fn lighting_matches(&self, expected: &Lighting, actual: &Lighting) -> bool {
        lighting_matches_firmware(&self.info.version, expected, actual)
    }

    pub fn side_lighting(&self) -> Option<SideLighting> {
        self.side.as_deref().and_then(SideLighting::from_raw)
    }

    pub fn matches(&self, profile: &Profile) -> bool {
        profile
            .base
            .iter()
            .all(|(id, binding)| self.binding(false, *id) == *binding)
            && profile
                .function
                .iter()
                .all(|(id, binding)| self.binding(true, *id) == *binding)
            && profile
                .lighting
                .as_ref()
                .is_none_or(|light| self.lighting_matches(light, &self.lighting()))
            && profile
                .side_lighting
                .as_ref()
                .is_none_or(|side| self.side_lighting().as_ref() == Some(side))
            && profile
                .settings
                .as_ref()
                .is_none_or(|s| *s == crate::features::KeyboardSettings::from_raw(&self.settings))
            && profile.custom_colors.iter().all(|(id, color)| {
                self.custom[*id as usize * 4 + 1..*id as usize * 4 + 4] == *color
            })
            && profile.macros.iter().all(|(id, m)| {
                self.macros
                    .get(id)
                    .map(|m| &m.actions)
                    .unwrap_or(&Vec::new())
                    == &m.actions
            })
    }
}

fn lighting_matches_firmware(version: &str, expected: &Lighting, actual: &Lighting) -> bool {
    // Verified on NUT75 1.40: Cross weave persists the brush RGB but runs its own colors.
    if version == "1.40" && expected.mode == 10 && actual.mode == 10 {
        let mut supported = expected.clone();
        supported.color = actual.color;
        supported.rainbow = actual.rainbow;
        supported.matches_device(actual)
    } else {
        expected.matches_device(actual)
    }
}

struct Session {
    protocol: Protocol<hidapi::HidDevice>,
    entry: DeviceEntry,
    // Keep the HID manager alive for the duration of a session.
    _api: hidapi::HidApi,
}

impl Session {
    fn connect(entry: DeviceEntry) -> Result<Self> {
        ensure!(
            entry.supported && is_nut75(&entry.name),
            "Thiết bị chưa được xác minh là NUT75. Dùng cáp USB để kết nối."
        );
        let api = hidapi::HidApi::new()?;
        let path = CString::new(entry.path_bytes.clone())?;
        let device = api.open_path(&path).context("Không mở được NUT75. Đóng driver web rồi kết nối lại; trên macOS kiểm tra quyền Giám sát đầu vào nếu hệ thống yêu cầu.")?;
        let mut descriptor = [0; 4096];
        let size = device.get_report_descriptor(&mut descriptor)?;
        let report_size = protocol::output_report_size(&descriptor[..size])?;
        if std::env::var_os("NUT75_TRACE_HID").is_some() {
            eprintln!(
                "NUT75 HID output: {report_size} byte. Descriptor: {:02X?}",
                &descriptor[..size]
            );
        }
        let protocol = Protocol {
            transport: device,
            report_size,
            timeout: Duration::from_millis(750),
        };
        let mut session = Self {
            protocol,
            entry,
            _api: api,
        };
        let info = session.info()?;
        if info.frame_version == 1 {
            session.protocol.timeout = Duration::from_secs(2);
        }
        Ok(session)
    }

    fn info(&self) -> Result<DeviceInfo> {
        DeviceInfo::parse(&self.protocol.transfer(protocol::GET_INFO, 48, None)?)
    }

    fn snapshot(&self) -> Result<Snapshot> {
        Ok(Snapshot {
            entry: self.entry.clone(),
            info: self.info()?,
            base: self.protocol.transfer(protocol::GET_KEYS, 512, None)?,
            function: self.protocol.transfer(protocol::GET_FN, 512, None)?,
            light: self.protocol.transfer(protocol::GET_LIGHT, 16, None)?,
            side: self
                .protocol
                .transfer(protocol::GET_SIDE, 24, None)
                .ok()
                .filter(|raw| SideLighting::from_raw(raw).is_some()),
            report_size: self.protocol.report_size,
            custom: self.protocol.transfer(protocol::GET_CUSTOM, 512, None)?,
            settings: self.protocol.transfer(protocol::GET_SETTINGS, 56, None)?,
            macros: self.read_macros()?,
        })
    }

    fn read_macros(&self) -> Result<BTreeMap<u8, crate::features::Macro>> {
        use crate::features::{Macro, MacroAction};
        let table = self.protocol.transfer(protocol::GET_MACRO, 400, None)?;
        let mut macros = BTreeMap::new();
        for id in 0..100 {
            let addr = u32::from_le_bytes(table[id * 4..id * 4 + 4].try_into().unwrap()) as usize;
            if addr == 0 {
                continue;
            }
            ensure!(
                (400..912).contains(&addr),
                "Địa chỉ macro ngoài bộ nhớ NUT75"
            );
            let header = self
                .protocol
                .transfer_at(protocol::GET_MACRO, addr, 4, None, true)?;
            let encoded = u16::from_le_bytes([header[0], header[1]]) as usize;
            ensure!(encoded.is_multiple_of(2), "Tiêu đề macro không hợp lệ");
            let count = encoded / 2;
            ensure!(addr + 4 + count * 4 <= 912, "Macro vượt bộ nhớ NUT75");
            let raw = if count == 0 {
                Vec::new()
            } else {
                self.protocol
                    .transfer_at(protocol::GET_MACRO, addr + 4, count * 4, None, true)?
            };
            let actions = raw
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| MacroAction {
                    delay: u16::from_le_bytes([b[0], b[1]]),
                    key: b[2],
                    status: b[3],
                })
                .collect();
            macros.insert(
                id as u8,
                Macro {
                    name: format!("Macro {}", id + 1),
                    actions,
                },
            );
        }
        Ok(macros)
    }

    fn apply(&self, profile: &Profile) -> Result<Snapshot> {
        profile.validate()?;
        // Read immediately before modifying: preserve hidden/advanced bindings and any changes
        // made on the keyboard since our previous read.
        let current = self.snapshot()?;
        ensure!(
            profile.side_lighting.is_none() || current.side.is_some(),
            "Firmware chưa trả về cấu hình đèn hai bên; chưa thể áp dụng vùng đèn này."
        );
        if !profile.macros.is_empty() {
            let mut macros = current.macros.clone();
            for (id, m) in &profile.macros {
                if m.actions.is_empty() {
                    macros.remove(id);
                } else {
                    macros.insert(*id, m.clone());
                }
            }
            let (table, body) = crate::features::macro_bytes(&macros)?;
            let current_actions: BTreeMap<_, _> = current
                .macros
                .iter()
                .map(|(id, m)| (*id, &m.actions))
                .collect();
            let new_actions: BTreeMap<_, _> =
                macros.iter().map(|(id, m)| (*id, &m.actions)).collect();
            if current_actions != new_actions {
                self.protocol.transfer_at(
                    protocol::SET_MACRO,
                    0,
                    400,
                    Some(&table),
                    body.is_empty(),
                )?;
                if !body.is_empty() {
                    self.protocol.transfer_at(
                        protocol::SET_MACRO,
                        400,
                        body.len(),
                        Some(&body),
                        true,
                    )?;
                }
                let readback = self.read_macros()?;
                ensure!(
                    readback.len() == macros.len()
                        && macros.iter().all(|(id, m)| readback
                            .get(id)
                            .is_some_and(|r| r.actions == m.actions)),
                    "Đọc lại macro không khớp"
                );
            }
        }
        for (get, set, raw, patch) in [
            (
                protocol::GET_CUSTOM,
                protocol::SET_CUSTOM,
                &current.custom,
                {
                    let mut data = current.custom.clone();
                    for (&id, color) in &profile.custom_colors {
                        let at = id as usize * 4;
                        data[at] = id;
                        data[at + 1..at + 4].copy_from_slice(color);
                    }
                    data
                },
            ),
            (
                protocol::GET_SETTINGS,
                protocol::SET_SETTINGS,
                &current.settings,
                {
                    let mut data = current.settings.clone();
                    if let Some(s) = &profile.settings {
                        s.merge_into(&mut data);
                    }
                    data
                },
            ),
        ] {
            if *raw != patch {
                self.protocol.transfer(set, patch.len(), Some(&patch))?;
                ensure!(
                    self.protocol.transfer(get, patch.len(), None)? == patch,
                    "Đọc lại cấu hình không khớp"
                );
            }
        }
        for (get, set, raw, patches) in [
            (
                protocol::GET_KEYS,
                protocol::SET_KEYS,
                &current.base,
                &profile.base,
            ),
            (
                protocol::GET_FN,
                protocol::SET_FN,
                &current.function,
                &profile.function,
            ),
        ] {
            if patches.is_empty() {
                continue;
            }
            let merged = model::merge_patches(raw, patches)?;
            if merged != *raw {
                self.protocol.transfer(set, 512, Some(&merged))?;
                let readback = self.protocol.transfer(get, 512, None)?;
                ensure!(
                    readback == merged,
                    "Đọc lại keymap không khớp. Có thể một phần cấu hình đã thay đổi; kết nối lại để kiểm tra."
                );
            }
        }
        if let Some(light) = &profile.lighting
            && !current.lighting_matches(light, &current.lighting())
        {
            let mut raw = current.light.clone();
            light.merge_into(&mut raw);
            self.protocol
                .transfer(protocol::SET_LIGHT, 16, Some(&raw))?;
            let mut matched = false;
            for attempt in 0..6 {
                if attempt > 0 {
                    std::thread::sleep(Duration::from_millis(100));
                }
                let readback = self.protocol.transfer(protocol::GET_LIGHT, 16, None)?;
                if current.lighting_matches(light, &Lighting::from_raw(&readback)) {
                    matched = true;
                    break;
                }
            }
            ensure!(matched, "Đọc lại RGB không khớp. Kết nối lại để kiểm tra.");
        }
        if let Some(side) = &profile.side_lighting
            && current.side_lighting().as_ref() != Some(side)
        {
            let mut raw = current.side.clone().context("Chưa đọc được đèn hai bên")?;
            side.merge_into(&mut raw);
            self.protocol.transfer(protocol::SET_SIDE, 24, Some(&raw))?;
            let readback = self.protocol.transfer(protocol::GET_SIDE, 24, None)?;
            ensure!(
                SideLighting::from_raw(&readback).as_ref() == Some(side),
                "Đọc lại đèn hai bên không khớp"
            );
        }
        self.snapshot()
    }
}

/// Read-only diagnostics for auxiliary LED support; never sends configuration writes.
pub fn probe_zones() -> Result<()> {
    let entry = enumerate()?
        .into_iter()
        .find(|e| e.supported)
        .context("Chưa thấy NUT75")?;
    let mut session = Session::connect(entry)?;
    session.protocol.timeout = Duration::from_millis(200);
    for (name, cmd) in [("Side", 0x1d), ("Light box", 0x1b)] {
        match session.protocol.transfer(cmd, 24, None) {
            Ok(raw) => println!("{name}: {raw:02X?}"),
            Err(e) => println!("{name}: {e:#}"),
        }
    }
    let custom = session.protocol.transfer(protocol::GET_CUSTOM, 512, None)?;
    let info = session.info()?;
    println!("Firmware {} / frame {}", info.version, info.frame_version);
    println!(
        "Main: {:02X?}",
        session.protocol.transfer(protocol::GET_LIGHT, 16, None)?
    );
    println!(
        "Settings: {:02X?}",
        session
            .protocol
            .transfer(protocol::GET_SETTINGS, 56, None)?
    );
    for (name, cmd) in [("Base", protocol::GET_KEYS), ("Fn", protocol::GET_FN)] {
        let raw = session.protocol.transfer(cmd, 512, None)?;
        for id in [13, 14, 15, 48, 60, 80, 85, 92, 104, 106] {
            println!("{name} {id}: {:02X?}", &raw[id * 4..id * 4 + 4]);
        }
    }
    println!("Custom Caps Lock: {:02X?}", &custom[48 * 4..49 * 4]);
    Ok(())
}

pub fn probe() -> Result<()> {
    let entry = enumerate()?
        .into_iter()
        .find(|entry| entry.supported)
        .context("Chưa tìm thấy NUT75. Kết nối USB-C và chọn chế độ USB.")?;
    let session = Session::connect(entry)?;
    let snapshot = session.snapshot()?;
    println!(
        "NUT75: {} | firmware {} | pin {}% | HID {} byte | keymap {} byte | Fn {} byte | RGB {:?}",
        snapshot.entry.name,
        snapshot.info.version,
        snapshot.info.battery,
        snapshot.report_size,
        snapshot.base.len(),
        snapshot.function.len(),
        snapshot.lighting()
    );
    Ok(())
}

/// Read actual LED engine output using the vendor's 64-byte receiver RGB565 query.
pub fn probe_led_colors() -> Result<()> {
    let entry = enumerate()?
        .into_iter()
        .find(|e| e.supported)
        .context("NUT75 not found")?;
    let session = Session::connect(entry)?;
    ensure!(
        session.entry.product_id == PRODUCT_RECEIVER && session.protocol.report_size == 64,
        "This read-only LED query requires the 64-byte NUT75 receiver"
    );
    let ids: Vec<u8> = (0u16..128).flat_map(u16::to_le_bytes).collect();
    let colors = session.protocol.query(0x3b, &ids)?;
    let mut counts = std::collections::BTreeMap::new();
    for c in colors.as_chunks::<2>().0 {
        let n = u16::from_be_bytes([c[0], c[1]]);
        let rgb = [
            ((n >> 11) & 31) as u8 * 8,
            ((n >> 5) & 63) as u8 * 4,
            (n & 31) as u8 * 8,
        ];
        *counts.entry(rgb).or_insert(0usize) += 1;
    }
    println!("Actual LED RGB565 counts: {counts:?}");
    Ok(())
}

/// Explicit, reversible lighting-only diagnostic. Never applies keymaps, settings or macros.
pub fn test_red_light(restore: Option<&std::path::Path>) -> Result<()> {
    let entry = enumerate()?
        .into_iter()
        .find(|e| e.supported)
        .context("NUT75 not found")?;
    let session = Session::connect(entry)?;
    let before = session.protocol.transfer(protocol::GET_LIGHT, 16, None)?;
    let data = if let Some(path) = restore {
        let (vid, pid, raw): (u16, u16, Vec<u8>) = serde_json::from_slice(&std::fs::read(path)?)?;
        ensure!(
            vid == session.entry.vendor_id && pid == session.entry.product_id && raw.len() == 16,
            "Lighting backup is for a different transport or is invalid"
        );
        raw
    } else {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let path = std::env::temp_dir().join(format!("nut75-light-backup-{stamp}.json"));
        let backup =
            serde_json::to_vec(&(session.entry.vendor_id, session.entry.product_id, &before))?;
        std::fs::write(&path, backup)?;
        println!("Restore backup: {}", path.display());
        let mut raw = before.clone();
        Lighting {
            mode: 1,
            color: [255, 0, 0],
            brightness: 5,
            speed: before[10],
            rainbow: false,
            direction: 0,
        }
        .merge_into(&mut raw);
        raw
    };
    session
        .protocol
        .transfer(protocol::SET_LIGHT, 16, Some(&data))?;
    let actual = session.protocol.transfer(protocol::GET_LIGHT, 16, None)?;
    ensure!(
        Lighting::from_raw(&data).matches_device(&Lighting::from_raw(&actual)),
        "Lighting diagnostic readback did not match"
    );
    println!("Lighting-only readback: {actual:02X?}");
    Ok(())
}

pub enum Command {
    Scan,
    Connect(DeviceEntry),
    Disconnect,
    Refresh,
    Apply(Profile),
}

pub enum Event {
    Devices(Vec<DeviceEntry>),
    Connected(Box<Snapshot>),
    Applied(Box<Snapshot>),
    Info(DeviceInfo),
    Disconnected,
    Error(String),
}

pub struct Worker {
    pub commands: mpsc::Sender<Command>,
    pub events: mpsc::Receiver<Event>,
}

impl Worker {
    pub fn start(context: eframe::egui::Context) -> Self {
        let (commands, incoming) = mpsc::channel();
        let (outgoing, events) = mpsc::channel();
        thread::Builder::new()
            .name("nut75-hid".into())
            .spawn(move || {
                let mut session: Option<Session> = None;
                loop {
                    let command = match incoming.recv_timeout(Duration::from_secs(5)) {
                        Ok(command) => Some(command),
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => None,
                    };
                    let error_summary = match command.as_ref() {
                        Some(Command::Connect(_)) => "Không kết nối được bàn phím",
                        Some(Command::Apply(_)) => "Không áp dụng được cấu hình",
                        Some(Command::Scan) => "Không tìm được thiết bị",
                        Some(Command::Refresh) | None => "Mất kết nối bàn phím",
                        Some(Command::Disconnect) => "Không ngắt kết nối được bàn phím",
                    };
                    let result: Result<Option<Event>> = (|| match command {
                        Some(Command::Scan) => Ok(Some(Event::Devices(enumerate()?))),
                        Some(Command::Connect(entry)) => {
                            session = None;
                            let connected = Session::connect(entry)?;
                            let snapshot = connected.snapshot()?;
                            session = Some(connected);
                            Ok(Some(Event::Connected(Box::new(snapshot))))
                        }
                        Some(Command::Disconnect) => {
                            session = None;
                            Ok(Some(Event::Disconnected))
                        }
                        Some(Command::Refresh) => {
                            let connected = session.as_ref().context("Chưa kết nối NUT75")?;
                            Ok(Some(Event::Connected(Box::new(connected.snapshot()?))))
                        }
                        Some(Command::Apply(profile)) => {
                            let connected = session.as_ref().context("Chưa kết nối NUT75")?;
                            Ok(Some(Event::Applied(Box::new(connected.apply(&profile)?))))
                        }
                        None => session
                            .as_ref()
                            .map(|connected| connected.info().map(|info| Some(Event::Info(info))))
                            .unwrap_or_else(|| {
                                enumerate().map(|entries| Some(Event::Devices(entries)))
                            }),
                    })();
                    let event = match result {
                        Ok(event) => event,
                        Err(error) => {
                            session = None;
                            eprintln!("{error:#}");
                            Some(Event::Error(error_summary.into()))
                        }
                    };
                    if let Some(event) = event {
                        if outgoing.send(event).is_err() {
                            break;
                        }
                        context.request_repaint();
                    }
                }
            })
            .expect("start HID worker");
        Self { commands, events }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_usb_id_is_not_a_transposed_hex_value() {
        assert_eq!(VENDOR, 3141);
        assert_eq!(PRODUCT_USB, 34828);
        assert_eq!(PRODUCT_RECEIVER, 65273);
    }

    #[test]
    fn shared_vendor_id_is_not_enough_to_treat_other_models_as_nut75() {
        assert!(is_nut75("NUT75"));
        assert!(is_nut75("NUT75 2.4G WIRELESS"));
        assert!(!is_nut75("NUT87"));
        assert!(!is_nut75("8K Wireless Receiver"));
    }

    #[test]
    fn bootloader_and_wrong_device_info_are_rejected() {
        let mut data = [0; 48];
        data[4..6].copy_from_slice(&VENDOR.to_le_bytes());
        data[6..8].copy_from_slice(&PRODUCT_USB.to_le_bytes());
        data[8] = 0x23;
        data[9] = 1;
        assert_eq!(DeviceInfo::parse(&data).unwrap().version, "1.23");
        data[32] = 1;
        assert!(DeviceInfo::parse(&data).is_err());
        data[32] = 0;
        data[6] = 0;
        assert!(DeviceInfo::parse(&data).is_err());
    }
}

#[cfg(test)]
mod lighting_capability_tests {
    use super::*;
    #[test]
    fn cross_weave_140_ignores_only_unsupported_brush_fields() {
        let expected = Lighting {
            mode: 10,
            ..Lighting::default()
        };
        let mut actual = expected.clone();
        actual.color = [255, 0, 0];
        actual.rainbow = true;
        assert!(lighting_matches_firmware("1.40", &expected, &actual));
        assert!(!lighting_matches_firmware("1.41", &expected, &actual));
        actual.brightness = 1;
        assert!(!lighting_matches_firmware("1.40", &expected, &actual));
    }

    #[test]
    fn supported_effects_detect_color_mode_changes_in_both_directions() {
        for &(mode, _) in crate::model::LIGHT_MODES {
            if matches!(mode, 0 | 6 | 8 | 10 | 23 | 128) {
                continue;
            }
            let single = Lighting {
                mode,
                rainbow: false,
                ..Lighting::default()
            };
            let rgb = Lighting {
                rainbow: true,
                ..single.clone()
            };
            assert!(
                !lighting_matches_firmware("1.40", &single, &rgb),
                "mode {mode}"
            );
            assert!(
                !lighting_matches_firmware("1.40", &rgb, &single),
                "mode {mode}"
            );
            assert!(lighting_matches_firmware("1.40", &single, &single));
            assert!(lighting_matches_firmware("1.40", &rgb, &rgb));
        }
    }
}
