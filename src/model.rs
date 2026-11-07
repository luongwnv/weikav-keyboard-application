use std::collections::BTreeMap;

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

pub type Binding = [u8; 4];
pub const DEFAULT: Binding = [0; 4];

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Colorway {
    Black,
    Silver,
    #[default]
    IceBlue,
    Orange,
    White,
    FrostSilver,
}

impl Colorway {
    pub const ALL: [Self; 6] = [
        Self::Black,
        Self::Silver,
        Self::IceBlue,
        Self::Orange,
        Self::White,
        Self::FrostSilver,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Black => "Black",
            Self::Silver => "Silver",
            Self::IceBlue => "Ice Blue",
            Self::Orange => "Vibrant Orange",
            Self::White => "White",
            Self::FrostSilver => "Frost Silver",
        }
    }
    pub fn case(self) -> [u8; 3] {
        match self {
            Self::Black => [48, 49, 53],
            Self::Silver => [186, 189, 194],
            Self::IceBlue => [160, 185, 202],
            Self::Orange => [234, 130, 56],
            Self::White => [237, 238, 237],
            Self::FrostSilver => [209, 215, 222],
        }
    }
    pub fn key_colors(self, id: u8) -> ([u8; 3], [u8; 3]) {
        let modifier = matches!(id, 5..=8 | 32 | 48 | 64 | 75 | 80..=82 | 85 | 87 | 104..=108);
        let accent = matches!(id, 0 | 76 | 83 | 88..=91);
        match self {
            Self::Black => (
                if accent {
                    [101, 53, 70]
                } else if modifier {
                    [75, 78, 82]
                } else {
                    [235, 235, 232]
                },
                if modifier || accent {
                    [241, 237, 239]
                } else {
                    [93, 48, 62]
                },
            ),
            Self::Silver => (
                if accent {
                    [59, 62, 66]
                } else if modifier {
                    [127, 131, 137]
                } else {
                    [234, 235, 235]
                },
                if modifier || accent {
                    [244, 244, 244]
                } else {
                    [68, 72, 79]
                },
            ),
            Self::IceBlue => (
                if modifier || id == 0 {
                    [159, 182, 196]
                } else {
                    [232, 234, 232]
                },
                if modifier || id == 0 {
                    [234, 240, 241]
                } else {
                    [75, 135, 169]
                },
            ),
            Self::Orange => (
                if accent {
                    [194, 78, 20]
                } else if modifier {
                    [212, 162, 108]
                } else {
                    [244, 239, 231]
                },
                if modifier || accent {
                    [255, 246, 229]
                } else {
                    [153, 79, 35]
                },
            ),
            Self::White => (
                if accent {
                    [163, 170, 166]
                } else {
                    [238, 239, 236]
                },
                if accent {
                    [248, 249, 245]
                } else {
                    [76, 86, 82]
                },
            ),
            Self::FrostSilver => (
                if modifier || accent {
                    [200, 210, 217]
                } else {
                    [234, 239, 241]
                },
                [102, 119, 132],
            ),
        }
    }
    pub fn photo(self) -> &'static [u8] {
        match self {
            Self::Black => include_bytes!("../assets/colorways/black.jpg"),
            Self::Silver => include_bytes!("../assets/colorways/silver.jpg"),
            Self::IceBlue => include_bytes!("../assets/colorways/ice-blue.jpg"),
            Self::Orange => include_bytes!("../assets/colorways/orange.jpg"),
            Self::White => include_bytes!("../assets/colorways/white.jpg"),
            Self::FrostSilver => include_bytes!("../assets/colorways/frost-silver.png"),
        }
    }
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalKey {
    pub value: u8,
    pub name: String,
    pub key_code: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    key_list: Vec<Vec<PhysicalKey>>,
}

pub fn layout() -> Vec<Vec<PhysicalKey>> {
    serde_json::from_str::<Layout>(include_str!("../assets/nut75-layout.json"))
        .expect("bundled official NUT75 layout is valid")
        .key_list
}

#[derive(Clone, Debug)]
pub struct KeyArea {
    pub id: u8,
    /// Coordinates in the original, unmodified 1000 × 400 driver image.
    pub rect: [f32; 4],
    pub key: PhysicalKey,
}

pub fn key_areas() -> Vec<KeyArea> {
    // Measured black key-well boundaries in the unmodified driver PNG.
    // Explicit row edges avoid accumulating a fractional unit-width error.
    const ROWS: &[(&[f32], f32, f32)] = &[
        (
            &[
                91., 142., 155., 206., 257., 309., 360., 373., 424., 475., 526., 577., 590., 641.,
                692., 743., 794., 807., 858., 859., 910.,
            ],
            40.,
            91.,
        ),
        (
            &[
                91., 142., 194., 245., 296., 347., 398., 449., 500., 552., 603., 654., 705., 756.,
                859., 910.,
            ],
            105.,
            156.,
        ),
        (
            &[
                91., 168., 219., 270., 321., 373., 424., 475., 526., 577., 628., 679., 731., 782.,
                859., 910.,
            ],
            156.,
            207.,
        ),
        (
            &[
                91., 181., 232., 283., 334., 385., 436., 488., 539., 590., 641., 692., 743., 859.,
                910.,
            ],
            207.,
            258.,
        ),
        (
            &[
                91., 206., 257., 309., 360., 411., 462., 513., 564., 615., 667., 718., 808., 859.,
            ],
            258.,
            309.,
        ),
        (
            &[
                91., 155., 219., 283., 603., 667., 731., 756., 807., 859., 910.,
            ],
            309.,
            360.,
        ),
    ];
    let top_spans = [
        (91., 142.),
        (155., 206.),
        (206., 257.),
        (257., 309.),
        (309., 360.),
        (373., 424.),
        (424., 475.),
        (475., 526.),
        (526., 577.),
        (590., 641.),
        (641., 692.),
        (692., 743.),
        (743., 794.),
        (807., 858.),
        (859., 910.),
    ];
    let mut areas = Vec::with_capacity(81);
    for (row, keys) in layout().into_iter().enumerate() {
        let (edges, top, bottom) = ROWS[row];
        for (column, key) in keys.into_iter().enumerate() {
            let (left, right) = if row == 0 {
                top_spans[column]
            } else if row == 5 && column >= 6 {
                (edges[column + 1], edges[column + 2])
            } else {
                (edges[column], edges[column + 1])
            };
            areas.push(KeyArea {
                id: key.value,
                rect: [left + 1., top + 1., right - left - 2., bottom - top - 2.],
                key,
            });
        }
    }
    areas
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Lighting {
    pub mode: u8,
    pub color: [u8; 3],
    pub brightness: u8,
    pub speed: u8,
    pub rainbow: bool,
    #[serde(default)]
    pub direction: u8,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            mode: 1,
            color: [132, 80, 255],
            brightness: 4,
            speed: 3,
            rainbow: false,
            direction: 0,
        }
    }
}

impl Lighting {
    pub fn matches_device(&self, actual: &Self) -> bool {
        self.mode == actual.mode
            && (self.mode == 0
                || (self.brightness == actual.brightness
                    && (matches!(self.mode, 1 | 128) || self.speed == actual.speed)
                    && (!light_has_direction(self.mode)
                        || self.normalized_direction() == actual.normalized_direction())
                    && (matches!(self.mode, 6 | 8 | 23 | 128)
                        || (self.rainbow == actual.rainbow
                            && (self.rainbow || self.color == actual.color)))))
    }
    pub fn normalized_direction(&self) -> u8 {
        match self.mode {
            10 if matches!(self.direction, 2 | 3) => self.direction,
            10 => 3,
            mode if light_has_direction(mode) && self.direction <= 1 => self.direction,
            _ => 0,
        }
    }
    pub fn from_raw(raw: &[u8]) -> Self {
        Self {
            // The vendor UI explicitly treats 20, 21 and 128 as custom lighting.
            // NUT75 firmware 1.40 reads mode 20 after SET mode 128.
            mode: if matches!(raw[0], 20 | 21) {
                128
            } else {
                raw[0]
            },
            color: [raw[1], raw[2], raw[3]],
            brightness: raw[9],
            speed: raw[10],
            rainbow: raw[8] != 0,
            direction: if light_has_direction(raw[0]) {
                raw[11]
            } else {
                0
            },
        }
    }

    /// Preserve all unedited fields supplied by the firmware.
    pub fn merge_into(&self, raw: &mut [u8]) {
        raw[0] = self.mode;
        raw[1..4].copy_from_slice(&self.color);
        raw[4] = 255;
        raw[8] = u8::from(self.rainbow);
        raw[9] = self.brightness;
        raw[10] = self.speed;
        if light_has_direction(self.mode) {
            raw[11] = self.normalized_direction();
        }
        raw[14] = 0xaa;
        raw[15] = 0x55;
    }
}

/// The vendor's separate side-light record, not the main key LED effect.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SideLighting {
    pub mode: u8,
    pub color: [u8; 3],
    pub brightness: u8,
    pub speed: u8,
    pub rainbow: bool,
}
impl SideLighting {
    pub fn from_raw(raw: &[u8]) -> Option<Self> {
        // Some NUT75 firmware ACKs this generic command with 24 zero bytes.
        // An ACK alone is not evidence that the zone is configurable.
        if raw.len() != 24
            || !(1..=6).contains(&raw[0])
            || raw[8] > 1
            || !(1..=6).contains(&raw[9])
            || !(1..=6).contains(&raw[10])
        {
            return None;
        }
        Some(Self {
            mode: raw[0],
            color: [raw[1], raw[2], raw[3]],
            brightness: raw[9],
            speed: raw[10],
            rainbow: raw[8] != 0,
        })
    }
    pub fn merge_into(&self, raw: &mut [u8]) {
        raw[0] = self.mode;
        raw[1..4].copy_from_slice(&self.color);
        raw[8] = u8::from(self.rainbow);
        raw[9] = self.brightness;
        raw[10] = self.speed;
    }
    pub fn preview(&self) -> Lighting {
        Lighting {
            mode: match self.mode {
                1 => 16,
                2 => 1,
                3 => 7,
                4 => 8,
                5 => 0,
                _ => 11,
            },
            color: self.color,
            brightness: self.brightness,
            speed: self.speed,
            rainbow: self.rainbow,
            direction: 0,
        }
    }
}
pub const SIDE_MODES: &[(u8, &str)] = &[
    (1, "Dòng chảy"),
    (2, "Sáng tĩnh"),
    (3, "Nhịp thở"),
    (4, "Quang phổ"),
    (5, "Tắt đèn"),
    (6, "Sóng màu"),
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u8,
    pub model: String,
    pub name: String,
    #[serde(default)]
    pub base: BTreeMap<u8, Binding>,
    #[serde(default)]
    pub function: BTreeMap<u8, Binding>,
    pub lighting: Option<Lighting>,
    #[serde(default)]
    pub side_lighting: Option<SideLighting>,
    #[serde(default)]
    pub macros: BTreeMap<u8, crate::features::Macro>,
    #[serde(default)]
    pub custom_colors: BTreeMap<u8, [u8; 3]>,
    #[serde(default)]
    pub settings: Option<crate::features::KeyboardSettings>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            schema_version: 1,
            model: "WEIKAV NUT75".into(),
            name: name.into(),
            base: BTreeMap::new(),
            function: BTreeMap::new(),
            lighting: None,
            side_lighting: None,
            macros: BTreeMap::new(),
            custom_colors: BTreeMap::new(),
            settings: None,
        }
    }

    pub fn patches(&self, function: bool) -> &BTreeMap<u8, Binding> {
        if function { &self.function } else { &self.base }
    }

    pub fn patches_mut(&mut self, function: bool) -> &mut BTreeMap<u8, Binding> {
        if function {
            &mut self.function
        } else {
            &mut self.base
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "Phiên bản hồ sơ không được hỗ trợ."
        );
        ensure!(
            self.model == "WEIKAV NUT75",
            "Hồ sơ này không dành cho WEIKAV NUT75."
        );
        ensure!(
            !self.name.trim().is_empty() && self.name.chars().count() <= 60,
            "Tên hồ sơ cần từ 1 đến 60 ký tự."
        );
        let valid: Vec<u8> = key_areas()
            .iter()
            .map(|k| k.id)
            .chain([13, 14, 15])
            .collect();
        for (function, patches) in [(false, &self.base), (true, &self.function)] {
            for (id, binding) in patches {
                ensure!(
                    valid.contains(id) && *id != 85,
                    "Vị trí phím không hợp lệ: {id}"
                );
                ensure!(
                    !(function && ((1..=12).contains(id) || [13, 14, 15].contains(id))),
                    "Vị trí Fn cố định: {id}"
                );
                ensure!(
                    valid_binding(*binding),
                    "Chức năng phím không được hỗ trợ: {binding:?}"
                );
            }
        }
        if let Some(light) = &self.lighting {
            ensure!(
                (1..=6).contains(&light.brightness) && (1..=6).contains(&light.speed),
                "Độ sáng và tốc độ cần nằm trong khoảng 1–6."
            );
            ensure!(
                LIGHT_MODES.iter().any(|(id, _)| *id == light.mode),
                "Hiệu ứng RGB không được hỗ trợ."
            );
        }
        if let Some(side) = &self.side_lighting {
            ensure!(
                (1..=6).contains(&side.mode)
                    && (1..=6).contains(&side.brightness)
                    && (1..=6).contains(&side.speed),
                "Cài đặt đèn hai bên ngoài phạm vi NUT75"
            );
        }
        crate::features::macro_bytes(&self.macros)?;
        for m in self.macros.values() {
            ensure!(m.name.chars().count() <= 60, "Tên macro quá dài");
            ensure!(
                m.actions
                    .iter()
                    .all(|a| matches!(a.status, 0x10 | 0x90 | 0x30 | 0xb0)),
                "Loại thao tác macro không hỗ trợ"
            );
        }
        if let Some(s) = &self.settings {
            ensure!(
                s.sleep <= 30
                    && (1..=5).contains(&s.debounce)
                    && [3, 5, 6].contains(&s.report_rate)
                    && s.system <= 3
                    && s.game_mode <= 1
                    && s.fn_switch <= 1
                    && s.nkro <= 1,
                "Cài đặt bàn phím ngoài phạm vi NUT75"
            );
        }
        ensure!(
            self.custom_colors.keys().all(|id| valid.contains(id)),
            "Vị trí LED không hợp lệ"
        );
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self> {
        ensure!(json.len() <= 64 * 1024, "Tệp hồ sơ quá lớn.");
        let profile: Self = serde_json::from_str(json).context("Tệp hồ sơ JSON không hợp lệ")?;
        profile.validate()?;
        Ok(profile)
    }
}

pub fn valid_binding(b: Binding) -> bool {
    match b {
        [0, 0, 0, 0] => true,
        [2, 0, key, 0] => (4..=0xa4).contains(&key) || (0xe0..=0xe7).contains(&key),
        [3, low, high, 0] => MEDIA
            .iter()
            .any(|(usage, _)| *usage == u16::from_le_bytes([low, high])),
        [1, param, value, 0] => MOUSE
            .iter()
            .any(|(binding, _)| binding == &[1, param, value, 0]),
        [6, id, mode, count] => id < 100 && mode <= 2 && (mode != 1 || count > 0),
        [7, modifier1, modifier2, key] => {
            (modifier1 == 0 || (0xe0..=0xe7).contains(&modifier1))
                && (modifier2 == 0 || (0xe0..=0xe7).contains(&modifier2))
                && (4..=0xe7).contains(&key)
        }
        [9, tap, hold, delay] => tap >= 4 && hold >= 4 && (1..=100).contains(&delay),
        [10, key, 0, 0] => key >= 4,
        [11, mode, first, second] => {
            (1..=4).contains(&mode) && first >= 4 && second >= 4 && first != second
        }
        [13, 0, 0, function] => (1..=68).contains(&function),
        _ => false,
    }
}

pub const MEDIA: &[(u16, &str)] = &[
    (0xe9, "Tăng âm lượng"),
    (0xea, "Giảm âm lượng"),
    (0xe2, "Tắt tiếng"),
    (0xcd, "Phát / tạm dừng"),
    (0xb5, "Bài tiếp theo"),
    (0xb6, "Bài trước"),
    (0xb7, "Dừng phát"),
];

pub const MOUSE: &[(Binding, &str)] = &[
    ([1, 1, 1, 0], "Chuột trái"),
    ([1, 1, 2, 0], "Chuột phải"),
    ([1, 1, 4, 0], "Chuột giữa"),
    ([1, 1, 8, 0], "Chuột quay lại"),
    ([1, 1, 16, 0], "Chuột tiến tới"),
    ([1, 3, 1, 0], "Cuộn chuột lên"),
    ([1, 3, 255, 0], "Cuộn chuột xuống"),
];

pub const FUNCTIONS: &[(u8, &str)] = &[
    (2, "Bluetooth 1"),
    (3, "Bluetooth 2"),
    (4, "Bluetooth 3"),
    (5, "Kết nối lại 2.4 GHz"),
    (6, "Đổi hàng F1–F12"),
    (7, "Xem pin"),
    (8, "Windows"),
    (9, "macOS"),
    (10, "Android"),
    (11, "Đổi hiệu ứng đèn"),
    (12, "Đổi màu đèn"),
    (13, "Tăng độ sáng"),
    (14, "Giảm độ sáng"),
    (15, "Tăng tốc độ đèn"),
    (16, "Giảm tốc độ đèn"),
    (17, "Tắt đèn"),
    (18, "Alt + Tab"),
    (19, "Win + E"),
    (20, "Win + Tab"),
    (21, "Win + H"),
    (22, "Khoá Win"),
    (23, "Đổi hiệu ứng đèn hai bên"),
    (24, "Đổi màu đèn hai bên"),
    (25, "Đổi độ sáng đèn hai bên"),
    (26, "Đổi tốc độ đèn hai bên"),
    (27, "Đổi hiệu ứng đèn cạnh núm"),
    (28, "Đổi tốc độ đèn cạnh núm"),
    (29, "Đổi màu đèn cạnh núm"),
    (30, "Ctrl + ↑"),
    (31, "Chế độ USB"),
    (32, "Đổi hướng đèn"),
    (33, "Alt + Esc"),
    (46, "Đổi chức năng con lăn"),
    (47, "Win + X"),
    (48, "Đổi Ctrl / Caps Lock"),
    (50, "Ctrl + ↓"),
    (51, "Hiệu ứng kế tiếp"),
    (52, "Hiệu ứng trước"),
    (53, "Win + D"),
    (58, "iOS"),
];

// Effect IDs taken from the official driver's translated effect list.
pub const LIGHT_MODES: &[(u8, &str)] = &[
    (0, "Tắt đèn"),
    (1, "Sáng tĩnh"),
    (2, "Chạm sáng"),
    (3, "Chạm tắt"),
    (4, "Sao lấp lánh"),
    (5, "Tuyết rơi"),
    (6, "Muôn sắc"),
    (7, "Nhịp thở"),
    (8, "Quang phổ"),
    (9, "Suối màu"),
    (10, "Đan sắc"),
    (11, "Sóng màu"),
    (12, "Núi màu"),
    (13, "Bùng sáng"),
    (14, "Sóng đôi"),
    (15, "Gợn sóng"),
    (16, "Dòng chảy"),
    (17, "Trùng điệp"),
    (18, "Mưa xiên"),
    (19, "Qua lại"),
    (23, "Chong chóng RGB"),
    (24, "Hội tụ"),
    (25, "Neon"),
    (128, "Tự chọn từng phím"),
];

pub fn light_has_direction(mode: u8) -> bool {
    matches!(mode, 10 | 11 | 12 | 16 | 18 | 23 | 25)
}

pub fn binding_label(b: Binding, default_name: &str) -> String {
    match b {
        [0, 0, 0, 0] => default_name.to_owned(),
        [2, 0, key, 0] => key_label(key),
        [3, low, high, 0] => MEDIA
            .iter()
            .find(|(usage, _)| *usage == u16::from_le_bytes([low, high]))
            .map(|(_, label)| (*label).to_owned())
            .unwrap_or_else(|| "Media (thiết bị)".into()),
        [1, ..] => MOUSE
            .iter()
            .find(|(binding, _)| *binding == b)
            .map(|(_, name)| (*name).to_owned())
            .unwrap_or_else(|| "Chuột (thiết bị)".into()),
        [6, id, ..] => format!("Macro {}", id + 1),
        [7, ..] => "Tổ hợp phím".into(),
        [9, ..] => "MT - Chạm / giữ".into(),
        [10, ..] => "TGL - Bật / tắt".into(),
        [11, ..] => "SOCD".into(),
        [13, ..] => "Chức năng bàn phím".into(),
        _ => "Chức năng từ thiết bị".into(),
    }
}

pub fn key_label(usage: u8) -> String {
    static NAMES: std::sync::OnceLock<BTreeMap<u16, String>> = std::sync::OnceLock::new();
    let names = NAMES.get_or_init(|| {
        layout()
            .into_iter()
            .flatten()
            .map(|key| (key.key_code, key.name))
            .collect()
    });
    if let Some(name) = names.get(&u16::from(usage)) {
        return name.clone();
    }
    match usage {
        70 => "Print Screen".into(),
        71 => "Scroll Lock".into(),
        72 => "Pause".into(),
        73 => "Insert".into(),
        101 => "Menu".into(),
        230 => "R-Alt".into(),
        231 => "R-Win".into(),
        _ => format!("HID {usage:02X}"),
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Vietnamese,
    English,
    ChineseSimplified,
    Japanese,
    Korean,
    French,
    German,
    Spanish,
}
impl Language {
    pub const ALL: [Self; 8] = [
        Self::Vietnamese,
        Self::English,
        Self::ChineseSimplified,
        Self::Japanese,
        Self::Korean,
        Self::French,
        Self::German,
        Self::Spanish,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Vietnamese => "Tiếng Việt",
            Self::English => "English",
            Self::ChineseSimplified => "简体中文",
            Self::Japanese => "日本語",
            Self::Korean => "한국어",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Spanish => "Español",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownDevice {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    pub active: usize,
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub colorway: Colorway,
    #[serde(default)]
    pub devices: Vec<KnownDevice>,
    #[serde(default)]
    pub hidden_devices: Vec<KnownDevice>,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub color_palette: Vec<[u8; 3]>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            active: 0,
            profiles: vec![Profile::new("Mặc định")],
            colorway: Colorway::default(),
            devices: Vec::new(),
            hidden_devices: Vec::new(),
            appearance: Appearance::default(),
            language: Language::default(),
            color_palette: Vec::new(),
        }
    }
}

pub fn data_path() -> std::path::PathBuf {
    directories::ProjectDirs::from("studio", "NUT75", "NUT Studio")
        .map(|dirs| dirs.config_dir().join("profiles.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("nut-studio-profiles.json"))
}

impl Library {
    pub fn device_hidden(&self, name: &str, vendor_id: u16, product_id: u16) -> bool {
        self.hidden_devices
            .iter()
            .any(|d| d.name == name && d.vendor_id == vendor_id && d.product_id == product_id)
    }
    /// Forget only discovery/history. Profiles and keyboard memory remain intact.
    pub fn forget_device(&mut self, device: &KnownDevice) {
        self.devices.retain(|d| d != device);
        if !self.hidden_devices.contains(device) {
            self.hidden_devices.push(device.clone());
        }
    }
    /// An explicit successful connection makes a forgotten device visible again.
    pub fn remember_device(&mut self, device: KnownDevice) -> bool {
        let previous = self.hidden_devices.len();
        self.hidden_devices.retain(|d| d != &device);
        let mut changed = previous != self.hidden_devices.len();
        if !self.devices.contains(&device) {
            self.devices.push(device);
            changed = true;
        }
        changed
    }
    pub fn load() -> Result<Self> {
        let path = data_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let lib: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        ensure!(
            !lib.profiles.is_empty()
                && lib.profiles.len() <= 100
                && lib.active < lib.profiles.len(),
            "Thư viện hồ sơ không hợp lệ."
        );
        for profile in &lib.profiles {
            profile.validate()?;
        }
        Ok(lib)
    }

    pub fn save(&self) -> Result<()> {
        let path = data_path();
        let parent = path.parent().context("Không tìm thấy thư mục cấu hình")?;
        std::fs::create_dir_all(parent)?;
        let temporary = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(self)?;
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        // std::fs::rename replaces existing files on both supported platforms.
        std::fs::rename(temporary, path).context("Không lưu được thư viện hồ sơ")
    }
}

pub fn merge_patches(raw: &[u8], patches: &BTreeMap<u8, Binding>) -> Result<Vec<u8>> {
    ensure!(raw.len() == 512, "Keymap cần có đúng 512 byte.");
    let mut merged = raw.to_vec();
    for (id, binding) in patches {
        if *id >= 128 {
            bail!("Vị trí phím nằm ngoài keymap.");
        }
        merged[usize::from(*id) * 4..usize::from(*id) * 4 + 4].copy_from_slice(binding);
    }
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_custom_aliases_and_effect_direction_defaults_are_normalized() {
        let mut raw = [0; 16];
        let custom = Lighting {
            mode: 128,
            ..Lighting::default()
        };
        custom.merge_into(&mut raw);
        for mode in [20, 21, 128] {
            raw[0] = mode;
            assert_eq!(Lighting::from_raw(&raw).mode, 128);
            assert!(custom.matches_device(&Lighting::from_raw(&raw)));
        }
        let cross = Lighting {
            mode: 10,
            direction: 0,
            ..Lighting::default()
        };
        cross.merge_into(&mut raw);
        assert_eq!(raw[11], 3);
        let wave = Lighting {
            mode: 11,
            direction: 3,
            ..Lighting::default()
        };
        wave.merge_into(&mut raw);
        assert_eq!(raw[11], 0);
    }
    #[test]
    fn side_lighting_requires_real_state_and_preserves_reserved_fields() {
        assert!(SideLighting::from_raw(&[0; 24]).is_none());
        assert!(SideLighting::from_raw(&[1; 16]).is_none());
        let mut raw = [42; 24];
        let side = SideLighting {
            mode: 2,
            color: [12, 34, 56],
            brightness: 4,
            speed: 3,
            rainbow: false,
        };
        side.merge_into(&mut raw);
        assert_eq!(SideLighting::from_raw(&raw), Some(side.clone()));
        assert_eq!(raw[4], 42);
        assert_eq!(raw[23], 42);
        let mut profile = Profile::new("Test");
        profile.side_lighting = Some(side);
        assert!(profile.validate().is_ok());
        profile.side_lighting.as_mut().unwrap().mode = 7;
        assert!(profile.validate().is_err());
    }
    #[test]
    fn hitbox_edges_follow_real_photo_key_wells_without_accumulated_drift() {
        let photo = image::load_from_memory(include_bytes!("../assets/nut75.png"))
            .unwrap()
            .into_rgb8();
        for area in key_areas() {
            let [x, y, w, h] = area.rect;
            for edge in [x, x + w] {
                let near_dark_border = (-2..=2).any(|delta| {
                    let pixel = photo.get_pixel(
                        (edge.round() as i32 + delta) as u32,
                        (y + h / 2.).round() as u32,
                    );
                    u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2]) < 3 * 85
                });
                assert!(
                    near_dark_border,
                    "{} edge {edge} drifts away from its photographed key well",
                    area.key.name
                );
            }
        }
    }

    #[test]
    fn old_profile_libraries_load_and_case_choice_does_not_modify_bindings() {
        let old = serde_json::json!({"active":0,"profiles":[Profile::new("Existing")]});
        let mut lib: Library = serde_json::from_value(old).unwrap();
        assert_eq!(lib.colorway, Colorway::IceBlue);
        assert!(lib.devices.is_empty());
        assert!(lib.appearance == Appearance::System);
        assert!(lib.language == Language::Vietnamese);
        let before = lib.profiles.clone();
        lib.colorway = Colorway::Orange;
        let roundtrip: Library =
            serde_json::from_str(&serde_json::to_string(&lib).unwrap()).unwrap();
        assert_eq!(roundtrip.colorway, Colorway::Orange);
        assert_eq!(before, roundtrip.profiles);
    }

    #[test]
    fn forgotten_devices_stay_hidden_after_reload_and_can_be_added_again() {
        let mut library = Library::default();
        let before = library.profiles.clone();
        let usb = KnownDevice {
            name: "NUT75".into(),
            vendor_id: 0x0c45,
            product_id: 0x880c,
        };
        let receiver = KnownDevice {
            name: "NUT75 2.4G".into(),
            vendor_id: 0x0c45,
            product_id: 0xfef9,
        };
        library.remember_device(usb.clone());
        library.remember_device(receiver.clone());
        library.forget_device(&usb);
        library.forget_device(&usb);
        assert_eq!(library.hidden_devices.len(), 1);
        let mut loaded: Library =
            serde_json::from_str(&serde_json::to_string(&library).unwrap()).unwrap();
        assert!(loaded.device_hidden(&usb.name, usb.vendor_id, usb.product_id));
        assert!(!loaded.device_hidden(&receiver.name, receiver.vendor_id, receiver.product_id));
        assert_eq!(loaded.devices.len(), 1);
        assert!(loaded.remember_device(usb.clone()));
        assert!(!loaded.device_hidden(&usb.name, usb.vendor_id, usb.product_id));
        assert_eq!(loaded.devices.len(), 2);
        assert_eq!(loaded.profiles, before);
    }

    #[test]
    fn official_layout_has_81_distinct_keys_and_three_rotary_actions() {
        let keys = key_areas();
        assert_eq!(keys.len(), 81);
        let mut ids: Vec<_> = keys.iter().map(|key| key.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 81);
        assert_eq!(keys.iter().find(|k| k.key.key_code == 4).unwrap().id, 49);
        assert_eq!(keys.iter().find(|k| k.key.key_code == 44).unwrap().id, 83);
    }

    #[test]
    fn patching_preserves_unknown_firmware_slots() {
        let original = vec![0x92; 512];
        let patches = BTreeMap::from([(49, [2, 0, 5, 0])]);
        let merged = merge_patches(&original, &patches).unwrap();
        assert_eq!(&merged[196..200], &[2, 0, 5, 0]);
        assert_eq!(&merged[..196], &original[..196]);
        assert_eq!(&merged[200..], &original[200..]);
    }

    #[test]
    fn import_rejects_other_models_invalid_slots_and_locked_fn_keys() {
        let mut p = Profile::new("Test");
        p.model = "NUT87".into();
        assert!(p.validate().is_err());
        p.model = "WEIKAV NUT75".into();
        p.base.insert(127, DEFAULT);
        assert!(p.validate().is_err());
        p.base.clear();
        p.function.insert(1, DEFAULT);
        assert!(p.validate().is_err());
        p.function.clear();
        p.base.insert(85, DEFAULT);
        assert!(p.validate().is_err());
        p.base.clear();
        p.base.insert(49, [2, 0, 5, 0]);
        assert_eq!(
            Profile::from_json(&serde_json::to_string(&p).unwrap()).unwrap(),
            p
        );
    }

    #[test]
    fn rgb_verification_checks_only_fields_supported_by_the_selected_effect() {
        let expected = Lighting {
            mode: 13,
            direction: 2,
            ..Lighting::default()
        };
        let mut actual = expected.clone();
        actual.direction = 0;
        assert!(expected.matches_device(&actual));
        actual.mode = 11;
        assert!(!expected.matches_device(&actual));
        let wave = Lighting {
            mode: 11,
            direction: 1,
            ..Lighting::default()
        };
        actual = wave.clone();
        actual.direction = 0;
        assert!(!wave.matches_device(&actual));
        let static_light = Lighting::default();
        actual = static_light.clone();
        actual.color = [255, 0, 0];
        assert!(!static_light.matches_device(&actual));
        let rgb = Lighting {
            mode: 13,
            rainbow: true,
            ..Lighting::default()
        };
        actual = rgb.clone();
        actual.color = [255, 255, 255];
        assert!(rgb.matches_device(&actual));
        actual.rainbow = false;
        assert!(!rgb.matches_device(&actual));
        let custom = Lighting {
            mode: 128,
            ..Lighting::default()
        };
        actual = custom.clone();
        actual.color = [0; 3];
        actual.speed = 0;
        assert!(custom.matches_device(&actual));
    }

    #[test]
    fn lighting_preserves_secondary_color_and_direction() {
        let mut raw = [77; 16];
        Lighting::default().merge_into(&mut raw);
        assert_eq!(&raw[5..8], &[77; 3]);
        assert_eq!(raw[11], 77);
        assert_eq!(&raw[14..], &[0xaa, 0x55]);
    }

    #[test]
    fn mouse_buttons_use_bit_masks_and_wheel_uses_signed_byte() {
        for (binding, name) in MOUSE {
            assert!(valid_binding(*binding));
            assert_eq!(binding_label(*binding, ""), *name);
        }
        assert!(valid_binding([1, 1, 16, 0]));
        assert!(valid_binding([1, 3, 255, 0]));
        assert!(!valid_binding([1, 1, 3, 0]));
        assert!(!valid_binding([1, 1, 5, 0]));
    }
}
