mod advanced;
mod color_picker;
mod controls;
mod home;
mod i18n;
mod lighting_zones;
mod macro_editor;
mod settings;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2, pos2,
    vec2,
};

use crate::{
    device::{Command, DeviceEntry, Event, Snapshot, Worker},
    model::{self, Binding, Colorway, DEFAULT, KeyArea, Library, Lighting, Profile},
    preview::{self, Ripple},
};

mod design;
use design::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Keys,
    Lighting,
    Macros,
    Profiles,
    Device,
    KeyboardSettings,
    Advanced,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Self::Keys => "PHÍM",
            Self::Lighting => "ĐÈN NỀN",
            Self::Macros => "MACRO",
            Self::Profiles => "Hồ sơ",
            Self::Device => "CÀI ĐẶT",
            Self::KeyboardSettings => "THIẾT BỊ",
            Self::Advanced => "NÂNG CAO",
        }
    }
    fn icon(self) -> u8 {
        match self {
            Self::Keys => 0,
            Self::Lighting => 1,
            Self::Macros => 5,
            Self::Profiles => 3,
            Self::Device => 4,
            Self::KeyboardSettings => 6,
            Self::Advanced => 7,
        }
    }
}

pub struct NutApp {
    worker: Worker,
    media_input: crate::media_input::Monitor,
    media_ready: bool,
    media_error: Option<String>,
    knob_pulse: Option<(u8, Instant)>,
    caps_lock: bool,
    library: Library,
    snapshot: Option<Box<Snapshot>>,
    devices: Vec<DeviceEntry>,
    keyboard: egui::TextureHandle,
    areas: Vec<KeyArea>,
    tab: Tab,
    home: bool,
    function: bool,
    selected: Option<u8>,
    show_labels: bool,
    category: usize,
    search: String,
    busy: bool,
    connect_open: bool,
    needs_save: bool,
    last_edit: Instant,
    notice: Option<(String, bool, Instant)>,
    save_error: bool,
    seen: HashSet<u8>,
    pressed: HashSet<u8>,
    last_key: String,
    profile_name: String,
    color_photos: Vec<egui::TextureHandle>,
    advanced_icons: [egui::TextureHandle; 4],
    appearance_open: bool,
    light_zone: u8,
    caps_color: [u8; 3],
    color_picker: color_picker::ColorPicker,
    preview_running: bool,
    preview_time: f64,
    ripples: Vec<Ripple>,
    macro_id: u8,
    recording: bool,
    last_record: Instant,
    record_modifiers: u8,
    advanced_kind: usize,
    advanced_keys: [u8; 3],
    advanced_delay: u16,
    advanced_mode: u8,
    macro_mode: u8,
    macro_repeat: u8,
    macro_action: Option<usize>,
    macro_table: bool,
    macro_add_open: bool,
    record_delays: bool,
    macro_undo: Option<(u8, crate::features::Macro)>,
    auto_connect: bool,
    connection_target: Option<String>,
}

impl NutApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        ctx.set_visuals(egui::Visuals::light());
        ctx.set_theme(egui::ThemePreference::Light);
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Inter".into(),
            egui::FontData::from_static(include_bytes!("../assets/InterVariable.ttf")).into(),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Inter".into());
        fonts.font_data.insert(
            "NotoCJK".into(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKsc-Regular.otf"
            ))
            .into(),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push("NotoCJK".into());
        }
        ctx.set_fonts(fonts);
        ctx.style_mut(|style| {
            style.visuals.panel_fill = bg();
            style.visuals.window_fill = surface();
            style.visuals.override_text_color = Some(ink());
            style.visuals.selection.bg_fill = purple();
            style.visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(229, 227, 236);
            style.visuals.widgets.inactive.weak_bg_fill = surface();
            style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, line());
            style.visuals.widgets.hovered.bg_fill = lilac();
            style.visuals.widgets.hovered.weak_bg_fill = lilac();
            style.visuals.widgets.hovered.bg_stroke =
                Stroke::new(1.0_f32, purple().gamma_multiply(0.4));
            style.visuals.widgets.active.bg_fill = purple();
            style.visuals.widgets.active.weak_bg_fill = lilac();
            for widget in [
                &mut style.visuals.widgets.inactive,
                &mut style.visuals.widgets.hovered,
                &mut style.visuals.widgets.active,
            ] {
                widget.corner_radius = 6.into();
            }
            style.spacing.button_padding = vec2(12., 8.);
            style.spacing.interact_size.y = 36.;
            style.spacing.scroll.floating = false;
            style.spacing.scroll.foreground_color = false;
            style.spacing.scroll.bar_width = 4.;
            style.spacing.scroll.bar_inner_margin = 12.;
            style.spacing.scroll.bar_outer_margin = 4.;
            style.spacing.scroll.handle_min_length = 36.;
            style.spacing.item_spacing = vec2(10., 12.);
            style
                .text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(16.));
            style
                .text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(14.));
            style
                .text_styles
                .insert(egui::TextStyle::Small, FontId::proportional(13.));
        });
        let image = image::load_from_memory(include_bytes!("../assets/nut75.png"))
            .expect("bundled NUT75 image")
            .into_rgba8();
        let keyboard = ctx.load_texture(
            "official-nut75",
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        let (library, notice) = match Library::load() {
            Ok(library) => (library, None),
            Err(error) => (
                Library::default(),
                Some((
                    format!(
                        "Không đọc được hồ sơ cũ: {error:#}. Tệp gốc được giữ nguyên; xuất hồ sơ mới để lưu."
                    ),
                    true,
                    Instant::now(),
                )),
            ),
        };
        let save_error = notice.is_some();
        let color_photos = Colorway::ALL
            .iter()
            .map(|variant| {
                let photo = image::load_from_memory(variant.photo())
                    .expect("bundled official colorway photo")
                    .into_rgba8();
                ctx.load_texture(
                    variant.name(),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [photo.width() as usize, photo.height() as usize],
                        photo.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                )
            })
            .collect();
        let advanced_icons = [
            ("MT", include_bytes!("../assets/advanced/mt.png").as_slice()),
            (
                "TGL",
                include_bytes!("../assets/advanced/tgl.png").as_slice(),
            ),
            (
                "SOCD",
                include_bytes!("../assets/advanced/socd.png").as_slice(),
            ),
            ("CB", include_bytes!("../assets/advanced/cb.png").as_slice()),
        ]
        .map(|(name, bytes)| {
            let image = image::load_from_memory(bytes)
                .expect("bundled vendor advanced icon")
                .into_rgba8();
            ctx.load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        });
        let worker = Worker::start(ctx.clone());
        let _ = worker.commands.send(Command::Scan);
        Self {
            profile_name: library.profiles[library.active].name.clone(),
            worker,
            media_input: crate::media_input::Monitor::start(ctx.clone()),
            media_ready: false,
            media_error: None,
            knob_pulse: None,
            caps_lock: crate::media_input::caps_lock(),
            library,
            notice,
            save_error,
            snapshot: None,
            devices: Vec::new(),
            keyboard,
            areas: model::key_areas(),
            tab: Tab::Keys,
            home: true,
            function: false,
            selected: None,
            show_labels: false,
            category: 0,
            search: String::new(),
            busy: true,
            connect_open: false,
            needs_save: false,
            last_edit: Instant::now(),
            seen: HashSet::new(),
            pressed: HashSet::new(),
            last_key: "—".into(),
            color_photos,
            advanced_icons,
            appearance_open: false,
            light_zone: 0,
            caps_color: [132, 80, 255],
            color_picker: color_picker::ColorPicker::default(),
            preview_running: true,
            preview_time: 0.,
            ripples: Vec::new(),
            macro_id: 0,
            recording: false,
            last_record: Instant::now(),
            record_modifiers: 0,
            advanced_kind: 0,
            advanced_keys: [4, 224, 6],
            advanced_delay: 400,
            advanced_mode: 3,
            macro_mode: 1,
            macro_repeat: 1,
            macro_action: None,
            macro_table: false,
            macro_add_open: false,
            record_delays: true,
            macro_undo: None,
            auto_connect: true,
            connection_target: None,
        }
    }

    fn profile(&self) -> &Profile {
        &self.library.profiles[self.library.active]
    }
    fn profile_mut(&mut self) -> &mut Profile {
        &mut self.library.profiles[self.library.active]
    }

    fn changed(&mut self) {
        self.needs_save = true;
        self.last_edit = Instant::now();
    }

    fn tell(&mut self, text: impl Into<String>, error: bool) {
        self.notice = Some((text.into(), error, Instant::now()));
    }

    fn send(&mut self, command: Command) {
        match &command {
            Command::Connect(entry) => self.connection_target = Some(entry.path.clone()),
            Command::Disconnect => {
                self.auto_connect = false;
                self.connection_target = None;
            }
            _ => {}
        }
        match self.worker.commands.send(command) {
            Ok(_) => self.busy = true,
            Err(_) => self.tell("Luồng kết nối HID đã dừng. Khởi động lại ứng dụng.", true),
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.worker.events.try_recv() {
            match event {
                Event::Devices(devices) => {
                    let newly_attached = devices
                        .iter()
                        .any(|e| e.supported && !self.devices.iter().any(|old| old.path == e.path));
                    self.devices = devices;
                    self.busy = false;
                    if self.auto_connect || newly_attached {
                        self.auto_connect = false;
                        if self.snapshot.is_none()
                            && let Some(entry) = self
                                .devices
                                .iter()
                                .filter(|e| {
                                    e.supported
                                        && !self.library.device_hidden(
                                            &e.name,
                                            e.vendor_id,
                                            e.product_id,
                                        )
                                })
                                .find(|e| {
                                    self.library.devices.iter().any(|d| {
                                        d.name == e.name
                                            && d.vendor_id == e.vendor_id
                                            && d.product_id == e.product_id
                                    })
                                })
                                .or_else(|| {
                                    self.devices.iter().find(|e| {
                                        e.supported
                                            && !self.library.device_hidden(
                                                &e.name,
                                                e.vendor_id,
                                                e.product_id,
                                            )
                                    })
                                })
                                .cloned()
                        {
                            self.send(Command::Connect(entry));
                        }
                    }
                }
                Event::Connected(snapshot) => {
                    self.connection_target = None;
                    let entry = &snapshot.entry;
                    if self.library.remember_device(model::KnownDevice {
                        name: entry.name.clone(),
                        vendor_id: entry.vendor_id,
                        product_id: entry.product_id,
                    }) {
                        self.changed();
                    }
                    self.snapshot = Some(snapshot);
                    self.busy = false;
                    self.connect_open = false;
                    self.tell("Đã kết nối", false);
                }
                Event::Applied(snapshot) => {
                    self.snapshot = Some(snapshot);
                    self.busy = false;
                    self.notice = None;
                }
                Event::Info(info) => {
                    if let Some(snapshot) = &mut self.snapshot {
                        snapshot.info = info;
                    }
                }
                Event::Disconnected => {
                    self.snapshot = None;
                    self.busy = false;
                    self.tell("Đã ngắt kết nối", false);
                }
                Event::Error(error) => {
                    self.connection_target = None;
                    self.snapshot = None;
                    self.busy = false;
                    self.tell(format!("{error}\nHồ sơ trên máy vẫn được giữ."), true);
                }
            }
        }
    }

    fn binding(&self, id: u8) -> Binding {
        self.profile()
            .patches(self.function)
            .get(&id)
            .copied()
            .or_else(|| {
                self.snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.binding(self.function, id))
            })
            .unwrap_or(DEFAULT)
    }

    fn key_name(&self, id: u8) -> String {
        self.areas
            .iter()
            .find(|area| area.id == id)
            .map(|area| area.key.name.clone())
            .unwrap_or_else(|| {
                match id {
                    13 => "Cuộn lên",
                    14 => "Cuộn xuống",
                    15 => "Nhấn con lăn",
                    _ => "Phím",
                }
                .into()
            })
    }

    fn locked(&self, id: u8) -> bool {
        id == 85 || (self.function && ((1..=12).contains(&id) || [13, 14, 15].contains(&id)))
    }

    fn assign(&mut self, binding: Binding) {
        if self.busy {
            return;
        }
        if let Some(id) = self.selected
            && !self.locked(id)
        {
            let function = self.function;
            let old = self.binding(id);
            if old[0] == 11 && old != binding {
                let partners: Vec<_> = self
                    .areas
                    .iter()
                    .filter(|area| {
                        area.id != id && !self.locked(area.id) && self.binding(area.id) == old
                    })
                    .map(|a| a.id)
                    .collect();
                for partner in partners {
                    self.profile_mut()
                        .patches_mut(function)
                        .insert(partner, DEFAULT);
                }
            }
            self.profile_mut().patches_mut(function).insert(id, binding);
            self.changed();
        }
    }

    fn save_library(&mut self) {
        // A corrupt library is kept intact; Export remains available to recover new work.
        if self.save_error {
            return;
        }
        match self.library.save() {
            Ok(_) => self.needs_save = false,
            Err(error) => {
                self.save_error = true;
                self.tell(
                    format!("Không lưu được hồ sơ: {error:#}. Bạn có thể xuất hồ sơ ra tệp JSON."),
                    true,
                );
            }
        }
    }

    fn export_profile(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(tr("Xuất hồ sơ NUT75"))
            .add_filter("NUT75 profile", &["json"])
            .set_file_name("nut75-profile.json")
            .save_file()
        {
            let result = serde_json::to_vec_pretty(self.profile())
                .map_err(anyhow::Error::from)
                .and_then(|bytes| Ok(std::fs::write(&path, bytes)?));
            match result {
                Ok(_) => self.tell(format!("Đã xuất hồ sơ: {}", path.display()), false),
                Err(error) => self.tell(format!("Không xuất được hồ sơ: {error}"), true),
            }
        }
    }

    fn import_profile(&mut self) {
        if self.library.profiles.len() >= 100 {
            self.tell("Thư viện đã có 100 hồ sơ.", true);
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_title(tr("Nhập hồ sơ NUT75"))
            .add_filter("NUT75 profile", &["json"])
            .pick_file()
        {
            let result = (|| -> anyhow::Result<Profile> {
                anyhow::ensure!(
                    std::fs::metadata(&path)?.len() <= 64 * 1024,
                    "Tệp hồ sơ quá lớn"
                );
                Profile::from_json(&std::fs::read_to_string(path)?)
            })();
            match result {
                Ok(profile) => {
                    self.profile_name = profile.name.clone();
                    self.library.profiles.push(profile);
                    self.library.active = self.library.profiles.len() - 1;
                    self.changed();
                    self.tell("Đã nhập hồ sơ", false);
                }
                Err(error) => self.tell(format!("Không nhập được hồ sơ: {error:#}"), true),
            }
        }
    }

    fn panel_open(&self) -> bool {
        match self.tab {
            Tab::Keys => self.selected.is_some(),
            Tab::Lighting | Tab::Profiles | Tab::Advanced => true,
            Tab::Macros => self.macro_action.is_some(),
            Tab::Device | Tab::KeyboardSettings => false,
        }
    }

    fn header(&mut self, ui: &mut egui::Ui, r: Rect) {
        let back = Rect::from_min_size(pos2(26., 18.), vec2(36., 36.));
        let response = ui.interact(back, egui::Id::new("back"), Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Về danh sách thiết bị"))
        });
        let center = back.center();
        let p = ui.painter();
        let stroke = Stroke::new(1.65_f32, ink());
        p.add(egui::Shape::line(
            vec![
                center + vec2(0., -5.),
                center + vec2(-5., 0.),
                center + vec2(0., 5.),
            ],
            stroke,
        ));
        p.line_segment([center - vec2(5., 0.), center + vec2(7., 0.)], stroke);
        if response.clicked() {
            self.go_home();
        }
        response.on_hover_cursor(egui::CursorIcon::PointingHand);
        text(
            ui.painter(),
            pos2(69., 36.),
            "NUT75",
            25.,
            ink(),
            Align2::LEFT_CENTER,
        );

        let profiles = Rect::from_min_size(pos2(r.right() - 196., 22.), vec2(30., 30.));
        let response = ui.interact(profiles, egui::Id::new("profiles"), Sense::click());
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Hồ sơ")));
        icon(
            ui.painter(),
            profiles.center(),
            3,
            if self.tab == Tab::Profiles {
                purple()
            } else {
                muted()
            },
            0.8,
        );
        if self.tab == Tab::Profiles {
            ui.painter().line_segment(
                [profiles.left_bottom(), profiles.right_bottom()],
                Stroke::new(1.4_f32, purple()),
            );
        }
        if response
            .on_hover_text(profile_display_name(&self.profile().name))
            .clicked()
        {
            self.tab = Tab::Profiles;
            self.selected = None;
        }
        ui.painter().line_segment(
            [pos2(r.right() - 150., 27.), pos2(r.right() - 150., 46.)],
            Stroke::new(1.0_f32, line()),
        );
        let connect = Rect::from_min_size(pos2(r.right() - 134., 20.), vec2(110., 32.));
        let response = ui.interact(connect, egui::Id::new("connection"), Sense::click());
        let label = if self.busy {
            "ĐANG KẾT NỐI"
        } else if self.snapshot.is_some() {
            "KẾT NỐI"
        } else {
            "+  KẾT NỐI"
        };
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Kết nối thiết bị"))
        });
        text(
            ui.painter(),
            connect.center(),
            label,
            12.,
            ink(),
            Align2::CENTER_CENTER,
        );
        if response.clicked() {
            self.connect_open = true;
            if !self.busy {
                self.send(Command::Scan);
            }
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, r: Rect) {
        let start = r.center().y - 150.;
        for (i, tab) in [
            Tab::Keys,
            Tab::Lighting,
            Tab::Macros,
            Tab::Advanced,
            Tab::KeyboardSettings,
            Tab::Device,
        ]
        .into_iter()
        .enumerate()
        {
            let item = Rect::from_min_size(pos2(30., start + i as f32 * 52.), vec2(148., 40.));
            let response = ui.interact(item, egui::Id::new(("nav", i)), Sense::click());
            let active = self.tab == tab;
            if active {
                ui.painter().rect_filled(item, 4., brand());
            } else if response.hovered() {
                ui.painter().rect_filled(item, 4., shade(242));
            }
            let color = if active { Color32::WHITE } else { ink() };
            let icon_box =
                Rect::from_center_size(pos2(item.left() + 19., item.center().y), vec2(24., 24.));
            sidebar_icon(ui.painter(), icon_box, tab.icon(), color);
            text(
                ui.painter(),
                pos2(item.left() + 38., item.center().y),
                tab.label(),
                14.,
                color,
                Align2::LEFT_CENTER,
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    true,
                    active,
                    tr(tab.label()),
                )
            });
            if response.clicked() {
                self.tab = tab;
                self.function = false;
                self.selected = None;
                self.pressed.clear();
                self.stop_recording();
            }
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
    }

    fn keyboard_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        let p = ui.painter_at(r);
        let viewport = Rect::from_min_max(
            r.min + vec2(28., 56.),
            r.max
                - vec2(
                    28.,
                    if self.tab == Tab::Lighting {
                        if self.light_zone == 0 { 360. } else { 216. }
                    } else {
                        64.
                    },
                ),
        );
        let width = viewport
            .width()
            .min(viewport.height() / 0.4)
            .clamp(1., 1100.);
        let image_rect = Rect::from_center_size(viewport.center(), vec2(width, width * 0.4));
        let scale = width / 1000.;
        let case = Rect::from_min_max(
            image_rect.min + vec2(79., 25.) * scale,
            image_rect.min + vec2(922., 386.) * scale,
        );
        for inset in (0..10).rev() {
            p.rect_filled(
                case.expand(inset as f32 * 1.5).translate(vec2(0., 8.)),
                14.,
                Color32::from_black_alpha(1),
            );
        }
        let mut light = self.current_lighting();
        if !self.solid_color_supported(light.mode) {
            light.rainbow = true;
        }
        let custom = self.custom_colors();
        if self.tab == Tab::Lighting || self.library.colorway != Colorway::IceBlue {
            preview::paint(
                &p,
                image_rect,
                &self.areas,
                self.library.colorway,
                if self.tab == Tab::Lighting {
                    Some((&light, self.preview_time, &self.ripples))
                } else {
                    None
                },
                self.keyboard.id(),
                &custom,
            );
        } else {
            p.image(
                self.keyboard.id(),
                image_rect,
                Rect::from_min_max(Pos2::ZERO, pos2(1., 1.)),
                Color32::WHITE,
            );
        }
        if self.tab == Tab::Lighting {
            if let Some(side) = self
                .profile()
                .side_lighting
                .clone()
                .or_else(|| self.snapshot.as_ref().and_then(|s| s.side_lighting()))
            {
                preview::paint_sides(&p, image_rect, &side.preview(), self.preview_time);
            }
            if self.light_zone == 1
                && let Some(area) = self.areas.iter().find(|a| a.id == 48)
            {
                let [x, y, w, h] = area.rect;
                p.rect_stroke(
                    Rect::from_min_size(image_rect.min + vec2(x, y) * scale, vec2(w, h) * scale)
                        .expand(3. * scale),
                    4. * scale,
                    Stroke::new(2. * scale, purple()),
                    StrokeKind::Outside,
                );
            }
        }
        // Separate status LED next to the wheel, as marked in the NUT75 manual.
        let indicator = Rect::from_min_size(
            image_rect.min + vec2(866., 276.) * scale,
            vec2(5., 15.) * scale,
        );
        p.rect_filled(indicator.expand(scale), 2. * scale, Color32::from_gray(65));
        if self.caps_lock {
            p.rect_filled(
                indicator.expand(3. * scale),
                3. * scale,
                green().gamma_multiply(0.18),
            );
            p.rect_filled(indicator, 1.5 * scale, Color32::from_rgb(220, 255, 232));
        } else {
            p.rect_filled(indicator, 1.5 * scale, Color32::from_gray(120));
        }
        ui.interact(
            indicator.expand(4. * scale),
            egui::Id::new("caps_indicator"),
            Sense::hover(),
        )
        .on_hover_text(if self.caps_lock {
            "Caps Lock: On"
        } else {
            "Caps Lock: Off"
        });
        let mut clicked = None;
        let mut paint_key = None;
        for area in &self.areas {
            let [x, y, w, h] = area.rect;
            let hit = Rect::from_min_size(image_rect.min + vec2(x, y) * scale, vec2(w, h) * scale);
            let response = ui.interact(
                hit,
                egui::Id::new(("key", area.id)),
                if matches!(self.tab, Tab::Keys | Tab::Lighting | Tab::Advanced) {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    format!("{} {}", tr("Phím"), area.key.name),
                )
            });
            let selected =
                matches!(self.tab, Tab::Keys | Tab::Advanced) && self.selected == Some(area.id);
            let patch = self.profile().patches(self.function).contains_key(&area.id);
            let color =
                if matches!(self.tab, Tab::Keys | Tab::Advanced) && self.seen.contains(&area.id) {
                    Some(green())
                } else if matches!(self.tab, Tab::Keys | Tab::Advanced)
                    && (selected || response.hovered() || patch)
                {
                    Some(purple())
                } else {
                    None
                };
            if let Some(color) = color {
                p.rect(
                    hit,
                    3. * scale,
                    color.gamma_multiply(if selected || self.seen.contains(&area.id) {
                        0.28
                    } else {
                        0.09
                    }),
                    Stroke::new(
                        if selected { 1.8_f32 } else { 1.1_f32 },
                        if selected { purple() } else { color },
                    ),
                    StrokeKind::Inside,
                );
            }
            if matches!(self.tab, Tab::Keys | Tab::Advanced) {
                let name = model::binding_label(self.binding(area.id), &area.key.name);
                response.clone().on_hover_text(format!(
                    "{} - {}{}",
                    area.key.name,
                    tr(&name),
                    if self.locked(area.id) {
                        tr(" (cố định)")
                    } else {
                        ""
                    }
                ));
                if patch {
                    p.circle_filled(hit.right_top() + vec2(-5., 5.), 2.3, purple());
                }
                if self.show_labels && self.binding(area.id) != DEFAULT {
                    p.rect_filled(
                        hit.shrink(2.),
                        3.,
                        if self.seen.contains(&area.id) {
                            green().gamma_multiply(0.3)
                        } else {
                            lilac()
                        },
                    );
                    let short: String = name.chars().take(if w > 80. { 13 } else { 5 }).collect();
                    text(
                        &p,
                        hit.center(),
                        &short,
                        10. * scale.max(0.8),
                        purple(),
                        Align2::CENTER_CENTER,
                    );
                }
                if response.clicked() {
                    clicked = Some(area.id);
                }
            } else if self.tab == Tab::Lighting && self.light_zone == 0 && response.clicked() {
                if light.mode == 128 {
                    paint_key = Some(area.id);
                }
                self.ripples.push(Ripple {
                    x: (x + w / 2. - 91.) / 819.,
                    y: (y + h / 2. - 40.) / 320.,
                    time: self.preview_time,
                });
            }
            if response.hovered() && matches!(self.tab, Tab::Keys | Tab::Lighting | Tab::Advanced) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        if let Some(id) = paint_key {
            self.profile_mut().custom_colors.insert(id, light.color);
            self.changed();
        }
        if let Some(id) = clicked {
            self.selected = Some(id);
            let b = self.binding(id);
            self.load_advanced_binding(b);
            if b[0] == 6 {
                self.macro_mode = b[2];
                self.macro_repeat = b[3].max(1);
            }
            self.search.clear();
        }

        if self.tab == Tab::Keys {
            let roller = Rect::from_min_max(
                image_rect.min + vec2(875., 273.) * scale,
                image_rect.min + vec2(913., 295.) * scale,
            );
            let response = ui.interact(roller, egui::Id::new("roller"), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Con lăn"))
            });
            if response.on_hover_text(tr("Con lăn")).clicked() {
                self.selected = Some(15);
                self.function = false;
            }
            if self.selected.is_some_and(|id| [13, 14, 15].contains(&id)) {
                p.rect_stroke(
                    roller.expand(2.),
                    3.,
                    Stroke::new(1.5_f32, purple()),
                    StrokeKind::Outside,
                );
            }
        }
        if self.tab == Tab::Keys {
            let y = (image_rect.bottom() + 28.).min(r.bottom() - 86.);
            let roller = Rect::from_min_max(
                image_rect.min + vec2(875., 273.) * scale,
                image_rect.min + vec2(913., 295.) * scale,
            );
            let pulse = self
                .knob_pulse
                .filter(|(_, time)| time.elapsed() < Duration::from_millis(650));
            let tested = [13, 14, 15].iter().any(|slot| self.seen.contains(slot));
            if tested {
                p.rect(
                    roller.expand(2. * scale),
                    3. * scale,
                    green().gamma_multiply(if pulse.is_some() { 0.4 } else { 0.18 }),
                    Stroke::new(2. * scale, green()),
                    StrokeKind::Outside,
                );
            }
            // Place the three test marks at the real wheel. These are passive visual
            // indicators, not separate buttons below the keyboard.
            for (slot, x) in [(14, -7.), (15, 19.), (13, 45.)] {
                if pulse.is_some_and(|(id, _)| id == slot) {
                    let center = roller.left_center() + vec2(x, 0.) * scale;
                    let color = if slot == 15 { Color32::WHITE } else { green() };
                    if slot == 15 {
                        p.circle_filled(center, 8. * scale, green());
                    }
                    if slot == 15 {
                        p.circle_stroke(center, 3. * scale, Stroke::new(1.5 * scale, color));
                    } else {
                        p.circle_filled(center, 6.5 * scale, surface());
                        p.circle_stroke(center, 6.5 * scale, Stroke::new(1.3 * scale, green()));
                        let direction = if slot == 14 { -1. } else { 1. };
                        p.line_segment(
                            [
                                center - vec2(3. * direction, 0.) * scale,
                                center + vec2(3. * direction, 0.) * scale,
                            ],
                            Stroke::new(1.6 * scale, color),
                        );
                        p.line(
                            [
                                center + vec2(0., -3.) * scale,
                                center + vec2(3. * direction, 0.) * scale,
                                center + vec2(0., 3.) * scale,
                            ]
                            .to_vec(),
                            Stroke::new(1.6 * scale, color),
                        );
                    }
                }
            }
            if pulse.is_some() {
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            }
            if let Some(error) = &self.media_error {
                ui.interact(
                    roller.expand(8.),
                    egui::Id::new("knob_input_status"),
                    Sense::hover(),
                )
                .on_hover_text(localize_message(error));
            }
            let y = y + 12.;
            let count = format!(
                "{} / 81",
                self.areas
                    .iter()
                    .filter(|a| self.seen.contains(&a.id))
                    .count()
            );
            let count_size = p
                .layout_no_wrap(count.clone(), FontId::proportional(13.), muted())
                .size();
            let group_width = count_size.x + 12. + 28.;
            let left = r.center().x - group_width / 2.;
            text(
                &p,
                pos2(left + count_size.x / 2., y),
                &count,
                13.,
                muted(),
                Align2::CENTER_CENTER,
            );
            let reset =
                Rect::from_center_size(pos2(left + count_size.x + 12. + 14., y), vec2(28., 28.));
            let response = ui.interact(reset, egui::Id::new("reset_key_test"), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Bắt đầu lại"))
            });
            if response.hovered() {
                p.circle_filled(reset.center(), 14., shade(242));
            }
            let glyph = p.layout_no_wrap("↻".into(), FontId::proportional(16.), muted());
            p.galley(
                reset.center() - glyph.mesh_bounds.center().to_vec2(),
                glyph,
                muted(),
            );
            if response.on_hover_text(tr("Bắt đầu lại")).clicked() {
                self.seen.clear();
                self.knob_pulse = None;
                self.pressed.clear();
                self.last_key = "—".into();
            }
        }
        if self.tab == Tab::Lighting {
            let width = (r.width() - 56.).min(700.);
            let mut top = image_rect.bottom() + 16.;
            if self.light_zone == 0 {
                let controls =
                    Rect::from_min_size(pos2(r.center().x - width / 2., top), vec2(width, 180.));
                top = self.lighting_controls_stage(ui, controls) + 10.;
            }
            let guide =
                Rect::from_min_size(pos2(r.center().x - width / 2., top), vec2(width, 224.));
            self.lighting_shortcut_guide(ui, guide);
        }
    }

    fn inspector(&mut self, ui: &mut egui::Ui, r: Rect) {
        let p = ui.painter();
        p.rect_filled(r, 0., surface());
        p.rect_filled(
            Rect::from_min_size(r.min, vec2(r.width(), 64.)),
            0.,
            shade(243),
        );
        let title = match self.tab {
            Tab::Keys => "Chức năng",
            Tab::Lighting => "Đèn nền",
            Tab::Profiles => "Hồ sơ",
            Tab::Macros => "Thao tác",
            Tab::Advanced => "Phím nâng cao",
            _ => "",
        };
        text(
            p,
            r.min + vec2(24., 32.),
            title,
            18.,
            ink(),
            Align2::LEFT_CENTER,
        );
        if icon_button_at(
            ui,
            Rect::from_min_size(r.right_top() + vec2(-52., 14.), vec2(36., 36.)),
            0,
            "Đóng bảng",
        )
        .clicked()
        {
            if self.tab == Tab::Macros {
                self.macro_action = None;
            } else {
                self.tab = Tab::Keys;
                self.selected = None;
                self.function = false;
            }
        }
        let inner = Rect::from_min_max(r.min + vec2(24., 84.), r.max - vec2(24., 76.));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.set_clip_rect(inner);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .id_salt(("inspector", self.tab as u8, self.light_zone))
            .show(&mut child, |ui| {
                let scroll = &ui.spacing().scroll;
                let gutter = scroll.bar_width + scroll.bar_inner_margin + scroll.bar_outer_margin;
                ui.set_width((inner.width() - gutter).max(0.));
                match self.tab {
                    Tab::Keys => self.key_inspector(ui),
                    Tab::Advanced => self.advanced_panel(ui),
                    Tab::Lighting => self.light_inspector(ui),
                    Tab::Profiles => self.profile_inspector(ui),
                    Tab::Macros => self.macro_inspector(ui),
                    _ => {}
                }
            });
    }

    fn key_inspector(&mut self, ui: &mut egui::Ui) {
        let Some(id) = self.selected else {
            return;
        };
        let roller = [13, 14, 15].contains(&id);
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.key_name(id)).size(18.).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if roller {
                    for (slot, name) in [(13, "Phải"), (15, "Nhấn"), (14, "Trái")] {
                        if ui.add(button(name, id == slot)).clicked() {
                            self.selected = Some(slot);
                        }
                    }
                } else {
                    for (function, name) in [(true, "Fn"), (false, "Mặc định")] {
                        if ui
                            .add(
                                egui::Button::new(RichText::new(tr(name)).size(13.).color(
                                    if self.function == function {
                                        purple()
                                    } else {
                                        muted()
                                    },
                                ))
                                .fill(if self.function == function {
                                    lilac()
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .frame(false),
                            )
                            .clicked()
                        {
                            self.function = function;
                            let b = self.binding(id);
                            self.load_advanced_binding(b);
                            if b[0] == 6 {
                                self.macro_mode = b[2];
                                self.macro_repeat = b[3].max(1);
                            }
                        }
                    }
                }
            });
        });
        ui.add_space(10.);
        if self.locked(id) {
            ui.label(
                RichText::new(tr("Chức năng này được cố định trên bàn phím."))
                    .size(14.)
                    .color(muted()),
            );
            return;
        }
        ui.add_enabled_ui(!self.busy, |ui| {
            if self.tab == Tab::Advanced {
                self.category = 4;
            } else {
                let categories = ["Phím", "Media", "Chuột", "Macro", "Nâng cao", "Bàn phím"];
                combo_box("action_category")
                    .selected_text(tr(categories[self.category]))
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for (category, name) in categories.iter().enumerate() {
                            dropdown_value(ui, &mut self.category, category, tr(name));
                        }
                    });
            }
            ui.add_space(8.);
            if ![3, 4].contains(&self.category) {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text(tr("Tìm kiếm"))
                        .desired_width(f32::INFINITY)
                        .margin(vec2(10., 8.)),
                );
                ui.add_space(8.);
            }
            if self.category == 3 {
                self.macro_bindings(ui);
                self.binding_reset(ui, id);
                return;
            }
            if self.category == 4 {
                self.advanced_inspector(ui);
                self.binding_reset(ui, id);
                return;
            }
            let mut options: Vec<(String, Binding)> = match self.category {
                5 => model::FUNCTIONS
                    .iter()
                    .map(|(id, name)| ((*name).to_owned(), [13, 0, 0, *id]))
                    .collect(),
                1 => model::MEDIA
                    .iter()
                    .map(|(usage, name)| {
                        let [lo, hi] = usage.to_le_bytes();
                        ((*name).into(), [3, lo, hi, 0])
                    })
                    .collect(),
                2 => model::MOUSE
                    .iter()
                    .map(|(binding, name)| ((*name).into(), *binding))
                    .collect(),
                _ => {
                    let mut options: Vec<_> = self
                        .areas
                        .iter()
                        .filter(|a| a.id != 85)
                        .map(|a| (a.key.name.clone(), [2, 0, a.key.key_code as u8, 0]))
                        .collect();
                    for usage in [70, 71, 72, 73, 101, 230, 231] {
                        options.push((model::key_label(usage), [2, 0, usage, 0]));
                    }
                    options
                }
            };
            let query = self.search.to_lowercase();
            options.retain(|(name, _)| tr(name).to_lowercase().contains(&query));
            ui.add_space(8.);
            egui::ScrollArea::vertical()
                .id_salt("actions")
                .max_height((ui.available_height() - 90.).max(140.))
                .show(ui, |ui| {
                    if self.category == 0 {
                        let width = (ui.available_width() - 12.) / 3.;
                        egui::Grid::new("keys_grid")
                            .num_columns(3)
                            .spacing([6., 6.])
                            .show(ui, |ui| {
                                for (i, (name, binding)) in options.iter().enumerate() {
                                    if ui
                                        .add_sized(
                                            [width, 32.],
                                            button(name, self.binding(id) == *binding),
                                        )
                                        .clicked()
                                    {
                                        self.assign(*binding);
                                    }
                                    if i % 3 == 2 {
                                        ui.end_row();
                                    }
                                }
                            });
                    } else {
                        for (name, binding) in options {
                            if ui
                                .add_sized(
                                    [ui.available_width(), 38.],
                                    button(&name, self.binding(id) == binding),
                                )
                                .clicked()
                            {
                                self.assign(binding);
                            }
                        }
                    }
                });
            ui.add_space(14.);
            if ui
                .add_sized(
                    [ui.available_width(), 32.],
                    button("Khôi phục mặc định", false),
                )
                .clicked()
            {
                self.assign(DEFAULT);
            }
            if self.profile().patches(self.function).contains_key(&id)
                && ui
                    .add(
                        egui::Button::new(RichText::new(tr("Hoàn tác")).size(13.).color(muted()))
                            .frame(false),
                    )
                    .clicked()
            {
                let function = self.function;
                self.profile_mut().patches_mut(function).remove(&id);
                self.changed();
            }
        });
    }

    fn custom_colors(&self) -> std::collections::BTreeMap<u8, [u8; 3]> {
        let mut colors = std::collections::BTreeMap::new();
        if let Some(s) = &self.snapshot {
            for (id, raw) in s.custom.as_chunks::<4>().0.iter().enumerate() {
                colors.insert(id as u8, [raw[1], raw[2], raw[3]]);
            }
        }
        colors.extend(self.profile().custom_colors.iter().map(|(k, v)| (*k, *v)));
        colors
    }

    fn current_lighting(&self) -> Lighting {
        self.profile()
            .lighting
            .clone()
            .or_else(|| self.snapshot.as_ref().map(|snapshot| snapshot.lighting()))
            .unwrap_or_default()
    }

    fn light_inspector(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 10.;
        if self.light_zone_picker(ui) {
            return;
        }
        let mut light = self.current_lighting();
        let before = light.clone();
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("Hiệu ứng")).size(14.));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, 1, "Chạy lại demo")
                    .on_hover_text(tr("Chạy lại demo"))
                    .clicked()
                {
                    self.preview_time = 0.;
                    self.ripples.clear();
                    self.preview_running = true;
                }
                if icon_button(
                    ui,
                    if self.preview_running { 2 } else { 3 },
                    "Phát / tạm dừng demo",
                )
                .on_hover_text(tr(if self.preview_running {
                    "Tạm dừng demo"
                } else {
                    "Phát demo"
                }))
                .clicked()
                {
                    self.preview_running = !self.preview_running;
                }
            });
        });
        ui.add_enabled_ui(!self.busy, |ui| {
            let width = (ui.available_width() - 16.) / 3.;
            egui::Grid::new("lighting_effects")
                .num_columns(3)
                .spacing([8., 8.])
                .show(ui, |ui| {
                    let height = if ui.ctx().content_rect().height() < 820. {
                        44.
                    } else {
                        50.
                    };
                    for (i, (id, name)) in model::LIGHT_MODES.iter().enumerate() {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, height), Sense::click());
                        let active = light.mode == *id;
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                true,
                                active,
                                tr(name),
                            )
                        });
                        ui.painter().rect(
                            rect,
                            6.,
                            if active {
                                lilac()
                            } else if response.hovered() {
                                shade(246)
                            } else {
                                surface()
                            },
                            Stroke::new(1.0_f32, if active { purple() } else { line() }),
                            StrokeKind::Inside,
                        );
                        effect_icon(
                            ui.painter(),
                            Rect::from_center_size(
                                pos2(rect.left() + 21., rect.center().y),
                                vec2(26., 26.),
                            ),
                            *id,
                            if active { purple() } else { muted() },
                        );
                        let galley = ui.painter().layout(
                            tr(name).into(),
                            FontId::proportional(14.),
                            if active { purple() } else { ink() },
                            width - 44.,
                        );
                        let top = rect.center().y - galley.size().y / 2.;
                        ui.painter()
                            .galley(pos2(rect.left() + 40., top), galley, ink());
                        if response.clicked() {
                            light.mode = *id;
                        }
                        if i % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });
        });
        light.direction = light.normalized_direction();
        if before != light {
            if before.mode != light.mode {
                self.preview_time = 0.;
                self.ripples.clear();
                self.preview_running = true;
            }
            light.brightness = light.brightness.clamp(1, 6);
            light.speed = light.speed.clamp(1, 6);
            self.profile_mut().lighting = Some(light);
            self.changed();
        }
    }

    fn profiles_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        let body = page_body(ui, r, "Hồ sơ");
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
        let mut selected = None;
        egui::ScrollArea::vertical()
            .max_height(r.height() - 190.)
            .show(&mut child, |ui| {
                for (i, profile) in self.library.profiles.iter().enumerate() {
                    let active = self.library.active == i;
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 94.), Sense::click());
                    ui.painter().rect(
                        rect,
                        12.,
                        if active { lilac() } else { surface() },
                        Stroke::new(
                            1.0_f32,
                            if active {
                                purple().gamma_multiply(0.4)
                            } else {
                                line()
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    icon(
                        ui.painter(),
                        pos2(rect.left() + 30., rect.center().y),
                        3,
                        if active { purple() } else { muted() },
                        1.2,
                    );
                    text(
                        ui.painter(),
                        pos2(rect.left() + 60., rect.top() + 30.),
                        &profile.name,
                        15.,
                        ink(),
                        Align2::LEFT_CENTER,
                    );
                    let details = format!(
                        "{} {}  -  {} macro{}",
                        profile.base.len() + profile.function.len(),
                        tr("phím"),
                        profile
                            .macros
                            .values()
                            .filter(|m| !m.actions.is_empty())
                            .count(),
                        if profile.lighting.is_some() {
                            "  -  RGB"
                        } else {
                            ""
                        }
                    );
                    text(
                        ui.painter(),
                        pos2(rect.left() + 60., rect.top() + 58.),
                        &details,
                        11.,
                        muted(),
                        Align2::LEFT_CENTER,
                    );
                    if active {
                        text(
                            ui.painter(),
                            pos2(rect.right() - 24., rect.center().y),
                            "Đang chọn",
                            10.,
                            purple(),
                            Align2::RIGHT_CENTER,
                        );
                    }
                    if response.clicked() && !self.busy {
                        selected = Some(i);
                    }
                    ui.add_space(6.);
                }
            });
        if let Some(i) = selected {
            self.library.active = i;
            self.profile_name = self.profile().name.clone();
            self.changed();
        }
        child.add_space(18.);
        if child
            .add_enabled(
                !self.busy && self.library.profiles.len() < 100,
                button("+  Hồ sơ mới", false),
            )
            .clicked()
        {
            let mut copy = self.profile().clone();
            copy.name = format!("{} {}", tr("Hồ sơ mới"), self.library.profiles.len() + 1);
            self.profile_name = copy.name.clone();
            self.library.profiles.push(copy);
            self.library.active = self.library.profiles.len() - 1;
            self.changed();
        }
    }

    fn profile_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(tr("Tên hồ sơ")).size(14.));
        ui.add_enabled(
            !self.busy,
            egui::TextEdit::singleline(&mut self.profile_name)
                .char_limit(60)
                .desired_width(f32::INFINITY)
                .margin(vec2(10., 8.)),
        );
        if ui
            .add_enabled(
                !self.busy
                    && !self.profile_name.trim().is_empty()
                    && self.profile_name.trim() != self.profile().name,
                button("Đổi tên", false),
            )
            .clicked()
        {
            self.profile_mut().name = self.profile_name.trim().to_owned();
            self.changed();
        }
        ui.add_space(28.);
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.busy, button("Nhập", false)).clicked() {
                self.import_profile();
            }
            if ui.add_enabled(!self.busy, button("Xuất", false)).clicked() {
                self.export_profile();
            }
        });
    }

    fn footer(&mut self, ui: &mut egui::Ui, r: Rect) {
        let battery = self
            .snapshot
            .as_ref()
            .filter(|s| s.entry.product_id != crate::device::PRODUCT_USB && s.info.battery <= 100)
            .map(|s| s.info.battery);
        let badge = Rect::from_min_size(
            pos2(30., r.center().y - 13.),
            vec2(if battery.is_some() { 78. } else { 38. }, 26.),
        );
        ui.painter().rect(
            badge,
            3.,
            surface(),
            Stroke::new(1.0_f32, line()),
            StrokeKind::Inside,
        );
        if let Some(level) = battery {
            let outline = Rect::from_min_size(badge.min + vec2(9., 9.), vec2(13., 8.));
            ui.painter().rect_stroke(
                outline,
                1.,
                Stroke::new(1.0_f32, green()),
                StrokeKind::Inside,
            );
            ui.painter().rect_filled(
                Rect::from_min_size(
                    outline.min + vec2(1.5, 1.5),
                    vec2(10. * f32::from(level) / 100., 5.),
                ),
                0.,
                green(),
            );
            text(
                ui.painter(),
                badge.min + vec2(30., 13.),
                &format!("{level}%"),
                10.,
                green(),
                Align2::LEFT_CENTER,
            );
        } else {
            icon(
                ui.painter(),
                badge.center(),
                0,
                if self.snapshot.is_some() {
                    green()
                } else {
                    muted()
                },
                0.75,
            );
        }
        let status = if self.busy {
            "Đang kết nối"
        } else if self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.info.charging)
        {
            "Đang sạc"
        } else if self.snapshot.is_some() {
            "Đã kết nối"
        } else {
            "Chưa kết nối"
        };
        let response = ui.interact(badge, egui::Id::new("connection_status"), Sense::click());
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(status)));
        if response.on_hover_text(tr(status)).clicked() {
            self.connect_open = true;
            if !self.busy {
                self.send(Command::Scan);
            }
        }
        let has_edits = !self.profile().base.is_empty()
            || !self.profile().function.is_empty()
            || self.profile().lighting.is_some()
            || !self.profile().macros.is_empty()
            || !self.profile().custom_colors.is_empty()
            || self.profile().settings.is_some();
        let applied = self
            .snapshot
            .as_ref()
            .is_some_and(|s| s.matches(self.profile()));
        if has_edits && !applied && self.snapshot.is_some() {
            let rect =
                Rect::from_min_size(pos2(r.right() - 162., r.center().y - 18.), vec2(134., 36.));
            let label = "Áp dụng";
            let response = ui.interact(
                rect,
                egui::Id::new("apply_hardware"),
                if self.busy {
                    Sense::hover()
                } else {
                    Sense::click()
                },
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, !self.busy, tr(label))
            });
            ui.painter().rect_filled(
                rect,
                4.,
                brand().gamma_multiply(if self.busy { 0.4 } else { 1. }),
            );
            text(
                ui.painter(),
                rect.center(),
                label,
                12.,
                Color32::WHITE,
                Align2::CENTER_CENTER,
            );
            if response.clicked() {
                self.send(Command::Apply(self.profile().clone()));
            }
        }
    }

    fn appearance_dialog(&mut self, ctx: &egui::Context) {
        if !self.appearance_open {
            return;
        }
        let modal = egui::Modal::new(egui::Id::new("appearance_modal"))
            .backdrop_color(Color32::from_black_alpha(28))
            .frame(
                egui::Frame::new()
                    .fill(surface())
                    .corner_radius(14.)
                    .inner_margin(28),
            )
            .show(ctx, |ui| {
                ui.set_width(460.);
                let corner = ui.max_rect().right_top() + vec2(4., -4.);
                if icon_button_at(
                    ui,
                    Rect::from_center_size(corner, vec2(32., 32.)),
                    5,
                    "Đóng ảnh",
                )
                .clicked()
                {
                    self.appearance_open = false;
                }
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.library.colorway.name())
                            .size(20.)
                            .strong(),
                    );
                });
                ui.add_space(16.);
                let texture = &self.color_photos[self.library.colorway.index()];
                let size = (ctx.content_rect().height() - 190.).clamp(220., 440.);
                ui.vertical_centered(|ui| {
                    ui.image((texture.id(), vec2(size, size)));
                });
            });
        if modal.should_close() {
            self.appearance_open = false;
        }
    }

    fn connection_dialog(&mut self, ctx: &egui::Context) {
        if !self.connect_open {
            return;
        }
        let mut connect = None;
        let modal = egui::Modal::new(egui::Id::new("connect_modal"))
            .backdrop_color(Color32::from_black_alpha(28))
            .frame(
                egui::Frame::new()
                    .fill(surface())
                    .corner_radius(14.)
                    .stroke(Stroke::new(1.0_f32, line()))
                    .inner_margin(28),
            )
            .show(ctx, |ui| {
                ui.set_width(420.);
                let corner = ui.max_rect().right_top() + vec2(4., -4.);
                if icon_button_at(
                    ui,
                    Rect::from_center_size(corner, vec2(32., 32.)),
                    5,
                    "Đóng kết nối",
                )
                .clicked()
                {
                    self.connect_open = false;
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr("Kết nối bàn phím")).size(20.).strong());
                });
                ui.add_space(24.);
                if self.devices.is_empty() {
                    ui.label(
                        RichText::new(tr(if self.busy {
                            "Đang tìm NUT75…"
                        } else {
                            "Cắm USB hoặc đánh thức bàn phím để kết nối."
                        }))
                        .size(14.)
                        .color(muted()),
                    );
                    ui.add_space(24.);
                }
                let mut action = None;
                egui::ScrollArea::vertical()
                    .id_salt("device_list")
                    .max_height(250.)
                    .show(ui, |ui| {
                        for entry in &self.devices {
                            let connected = self
                                .snapshot
                                .as_ref()
                                .is_some_and(|s| s.entry.path == entry.path);
                            egui::Frame::new()
                                .fill(bg())
                                .corner_radius(8.)
                                .inner_margin(16)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let (r, response) =
                                            ui.allocate_exact_size(vec2(96., 44.), Sense::hover());
                                        if entry.supported {
                                            let thumbnail =
                                                Rect::from_center_size(r.center(), vec2(96., 38.4));
                                            preview::paint(
                                                ui.painter(),
                                                thumbnail,
                                                &self.areas,
                                                self.library.colorway,
                                                None,
                                                self.keyboard.id(),
                                                &Default::default(),
                                            );
                                            response.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::Image,
                                                    true,
                                                    "WEIKAV NUT75",
                                                )
                                            });
                                        } else {
                                            icon(ui.painter(), r.center(), 0, muted(), 0.9);
                                        }
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&entry.name).size(15.).strong());
                                            ui.label(
                                                RichText::new(tr(if connected {
                                                    "Đã kết nối"
                                                } else if entry.supported {
                                                    if entry.product_id
                                                        == crate::device::PRODUCT_USB
                                                    {
                                                        "USB-C"
                                                    } else {
                                                        "2.4 GHz"
                                                    }
                                                } else {
                                                    "Chưa hỗ trợ thiết bị này"
                                                }))
                                                .size(13.)
                                                .color(muted()),
                                            );
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add_enabled(
                                                        !self.busy && entry.supported && !connected,
                                                        button(
                                                            if connected {
                                                                "Đã kết nối"
                                                            } else {
                                                                "Kết nối"
                                                            },
                                                            !connected,
                                                        )
                                                        .min_size(vec2(104., 36.)),
                                                    )
                                                    .clicked()
                                                {
                                                    action = Some(entry.clone());
                                                }
                                            },
                                        );
                                    });
                                });
                            ui.add_space(8.);
                        }
                    });
                connect = action;
                ui.add_space(20.);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.busy,
                            button("Quét lại", false).min_size(vec2(104., 36.)),
                        )
                        .clicked()
                    {
                        self.send(Command::Scan);
                    }
                    if self.busy {
                        ui.spinner();
                    }
                    if self.snapshot.is_some() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    !self.busy,
                                    button("Ngắt kết nối", false).min_size(vec2(120., 36.)),
                                )
                                .clicked()
                            {
                                self.send(Command::Disconnect);
                            }
                        });
                    }
                });
            });
        if modal.should_close() {
            self.connect_open = false;
        }
        if let Some(entry) = connect {
            self.open_device(Some(entry));
        }
    }

    fn test_key(&mut self, usage: u16, pressed: bool) {
        for area in &self.areas {
            if area.key.key_code == usage {
                if pressed {
                    self.pressed.insert(area.id);
                    self.seen.insert(area.id);
                    self.last_key = area.key.name.clone();
                } else {
                    self.pressed.remove(&area.id);
                }
            }
        }
    }

    fn keyboard_events(&mut self, ctx: &egui::Context) {
        if self.home
            || !matches!(self.tab, Tab::Keys | Tab::Advanced)
            || self.connect_open
            || self.appearance_open
        {
            return;
        }
        if !ctx.input(|i| i.focused) || ctx.wants_keyboard_input() {
            self.pressed.clear();
            return;
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                ..
            } = event
            {
                if repeat {
                    continue;
                }
                let physical = physical_key.unwrap_or(key);
                if let Some(usage) = usage_for_key(physical) {
                    for area in &self.areas {
                        if area.key.key_code == u16::from(usage) {
                            if pressed {
                                self.pressed.insert(area.id);
                                self.seen.insert(area.id);
                                self.last_key = area.key.name.clone();
                            } else {
                                self.pressed.remove(&area.id);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for NutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        design::apply(ctx, &self.library);
        self.process_events();
        let testing = !self.home
            && matches!(self.tab, Tab::Keys | Tab::Advanced)
            && ctx.input(|i| i.focused)
            && !self.connect_open;
        let caps_lock = crate::media_input::caps_lock();
        if caps_lock != self.caps_lock {
            self.caps_lock = caps_lock;
            if testing {
                self.seen.insert(48);
            }
        }
        self.media_input.set_active(testing);
        while let Ok(event) = self.media_input.events.try_recv() {
            match event {
                crate::media_input::InputEvent::Pulse(slot) if testing => {
                    self.media_error = None;
                    self.media_ready = true;
                    self.knob_pulse = Some((slot, Instant::now()));
                    self.seen.insert(slot);
                }
                #[cfg(target_os = "macos")]
                crate::media_input::InputEvent::Key { usage, pressed }
                    if testing && !ctx.wants_keyboard_input() =>
                {
                    self.test_key(usage, pressed);
                }
                crate::media_input::InputEvent::Ready => {
                    self.media_ready = true;
                    self.media_error = None;
                }
                crate::media_input::InputEvent::Unavailable(error) => {
                    self.media_ready = false;
                    self.media_error = Some(error);
                }
                _ => {}
            }
        }
        #[cfg(target_os = "windows")]
        if testing && !ctx.wants_keyboard_input() {
            for (usage, pressed) in crate::media_input::modifier_states() {
                self.test_key(usage, pressed);
            }
        }
        self.keyboard_events(ctx);
        if !self.home {
            self.macro_events(ctx);
        }
        if !self.home && self.tab == Tab::Lighting && self.preview_running {
            self.preview_time += f64::from(ctx.input(|i| i.stable_dt).min(0.1));
            self.ripples
                .retain(|pulse| self.preview_time - pulse.time < 3.);
            if self.light_zone == 2 || !matches!(self.current_lighting().mode, 0 | 1) {
                ctx.request_repaint_after(Duration::from_millis(33));
            }
        }
        if self.needs_save && self.last_edit.elapsed() >= Duration::from_millis(450) {
            self.save_library();
        }
        if self.needs_save && !self.save_error {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg()))
            .show(ctx, |ui| {
                let r = ui.max_rect();
                if self.home {
                    self.home_stage(ui, r);
                    return;
                }
                let panel_width = if self.tab == Tab::Lighting {
                    if r.width() >= 1250. { 480. } else { 444. }
                } else if self.panel_open() {
                    340.
                } else {
                    0.
                };
                let workspace =
                    Rect::from_min_max(r.min, pos2(r.right() - panel_width, r.bottom()));
                self.header(ui, workspace);
                let content = Rect::from_min_max(pos2(0., 84.), pos2(r.right(), r.bottom() - 56.));
                let key_detail = self.tab == Tab::Keys && self.selected.is_some();
                if !key_detail {
                    self.sidebar(ui, content);
                }
                let stage = Rect::from_min_max(
                    pos2(if key_detail { 24. } else { 190. }, content.top()),
                    pos2(workspace.right(), content.bottom()),
                );
                match self.tab {
                    Tab::Profiles => self.profiles_stage(ui, stage),
                    Tab::Device => self.device_stage(ui, stage),
                    Tab::KeyboardSettings => self.keyboard_settings_stage(ui, stage),
                    Tab::Advanced => self.advanced_stage(ui, stage),
                    Tab::Macros => self.macro_stage(ui, stage),
                    _ => self.keyboard_stage(ui, stage),
                }
                if panel_width > 0. {
                    self.inspector(ui, Rect::from_min_max(pos2(workspace.right(), 0.), r.max));
                }
                self.footer(ui, Rect::from_min_max(pos2(0., r.bottom() - 56.), r.max));
            });
        if let Some((message, error, created)) = self.notice.clone() {
            if error || created.elapsed() < Duration::from_secs(3) {
                egui::Area::new(egui::Id::new("notice"))
                    .anchor(
                        if error
                            && !self.home
                            && matches!(self.tab, Tab::Keys | Tab::Lighting | Tab::Advanced)
                        {
                            Align2::LEFT_TOP
                        } else {
                            Align2::RIGHT_TOP
                        },
                        if error
                            && !self.home
                            && matches!(self.tab, Tab::Keys | Tab::Lighting | Tab::Advanced)
                        {
                            vec2(
                                if self.tab == Tab::Keys && self.selected.is_some() {
                                    52.
                                } else {
                                    218.
                                },
                                104.,
                            )
                        } else {
                            vec2(-24., 84.)
                        },
                    )
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        let frame = egui::Frame::new()
                            .fill(if error {
                                Color32::from_rgb(255, 242, 239)
                            } else {
                                lilac()
                            })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if error {
                                    Color32::from_rgb(244, 210, 201)
                                } else {
                                    Color32::from_rgb(222, 207, 252)
                                },
                            ))
                            .corner_radius(10.)
                            .inner_margin(14)
                            .show(ui, |ui| {
                                let width = if error { 400. } else { 260. };
                                ui.set_width(width);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(localize_message(&message)).size(14.).color(
                                            if error {
                                                Color32::from_rgb(159, 65, 49)
                                            } else {
                                                purple()
                                            },
                                        ),
                                    )
                                    .wrap(),
                                );
                            });
                        let close = Rect::from_center_size(
                            frame.response.rect.right_top() + vec2(-4., 4.),
                            vec2(28., 28.),
                        );
                        ui.allocate_rect(close, Sense::hover());
                        if icon_button_at(ui, close, 5, tr("Đóng thông báo")).clicked() {
                            self.notice = None;
                        }
                    });
                ctx.request_repaint_after(Duration::from_secs(1));
            } else {
                self.notice = None;
            }
        }
        self.connection_dialog(ctx);
        self.appearance_dialog(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_recording();
        if self.needs_save {
            self.save_library();
        }
    }
}

fn page_body(ui: &egui::Ui, r: Rect, title: &str) -> Rect {
    let inner = r.shrink2(vec2(28., 28.));
    text(
        ui.painter(),
        inner.min + vec2(0., 16.),
        title,
        24.,
        ink(),
        Align2::LEFT_CENTER,
    );
    Rect::from_min_max(inner.min + vec2(0., 56.), inner.max)
}

fn icon_button(ui: &mut egui::Ui, kind: u8, name: &str) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(vec2(36., 36.), Sense::hover());
    icon_button_at(ui, rect, kind, name)
}

fn icon_button_at(ui: &mut egui::Ui, rect: Rect, kind: u8, name: &str) -> egui::Response {
    let response = ui.interact(rect, ui.make_persistent_id(name), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(name)));
    let p = ui.painter();
    let c = rect.center();
    let control_color = if response.hovered() { ink() } else { muted() };
    if response.hovered() {
        p.circle_filled(c, 16., shade(233));
    }
    match kind {
        5 | 0 => {
            p.circle_filled(
                c,
                12.,
                if kind == 5 {
                    surface()
                } else {
                    Color32::from_rgba_unmultiplied(
                        130,
                        130,
                        145,
                        if response.hovered() { 40 } else { 18 },
                    )
                },
            );
            p.circle_stroke(
                c,
                12.,
                Stroke::new(0.75_f32, Color32::from_rgba_unmultiplied(130, 130, 145, 90)),
            );
            let stroke = Stroke::new(1.3_f32, Color32::from_gray(125));
            p.line_segment([c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], stroke);
            p.line_segment([c + vec2(-4.5, 4.5), c + vec2(4.5, -4.5)], stroke);
        }
        1 => {
            let glyph = p.layout_no_wrap("↻".into(), FontId::proportional(18.), control_color);
            p.galley(
                c - glyph.mesh_bounds.center().to_vec2(),
                glyph,
                control_color,
            );
        }
        2 => {
            for x in [-4.5, 4.5] {
                p.rect_filled(
                    Rect::from_center_size(c + vec2(x, 0.), vec2(3., 16.)),
                    1.,
                    control_color,
                );
            }
        }
        _ => {
            p.add(egui::Shape::convex_polygon(
                vec![c + vec2(-6., -9.), c + vec2(9., 0.), c + vec2(-6., 9.)],
                control_color,
                Stroke::NONE,
            ));
        }
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tr(name))
}

fn effect_icon(p: &egui::Painter, r: Rect, mode: u8, color: Color32) {
    let c = r.center();
    let _selected = color == purple();
    let color = match mode {
        0 | 3 | 14 | 19 => Color32::from_rgb(242, 166, 190),
        1 | 4 | 13 => Color32::from_rgb(243, 205, 132),
        2 | 16 | 24 => Color32::from_rgb(151, 220, 178),
        5 | 9 | 11 | 18 => Color32::from_rgb(137, 210, 230),
        _ => Color32::from_rgb(180, 161, 237),
    };
    let stroke = Stroke::new(1.6_f32, color);
    let line = |a: Vec2, b: Vec2| {
        p.line_segment([c + a, c + b], stroke);
    };
    match mode {
        0 => {
            p.circle_stroke(c, 9., stroke);
            line(vec2(-6., 6.), vec2(6., -6.));
        }
        1 | 13 => {
            p.circle_stroke(c, if mode == 1 { 4. } else { 3. }, stroke);
            for i in 0..8 {
                let v = Vec2::angled(i as f32 * std::f32::consts::TAU / 8.);
                line(v * 7., v * 11.);
            }
        }
        2 | 3 => {
            p.rect_stroke(r.shrink(5.), 3., stroke, StrokeKind::Inside);
            line(vec2(-3., 0.), vec2(3., 0.));
            if mode == 2 {
                line(vec2(0., -3.), vec2(0., 3.));
            }
            for a in [-1., 1.] {
                line(vec2(a * 8., -8.), vec2(a * 11., -11.));
            }
        }
        4 => {
            for (x, y, s) in [(-6., -4., 3.), (6., 5., 4.), (6., -8., 2.)] {
                line(vec2(x - s, y), vec2(x + s, y));
                line(vec2(x, y - s), vec2(x, y + s));
            }
        }
        5 => {
            for i in 0..3 {
                let v = Vec2::angled(i as f32 * std::f32::consts::PI / 3.);
                line(-v * 10., v * 10.);
                for a in [-1., 1.] {
                    let tip = v * 7. * a;
                    line(tip, tip - v.rot90() * 3. - v * 3. * a);
                    line(tip, tip + v.rot90() * 3. - v * 3. * a);
                }
            }
        }
        6 => {
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.;
                let color = Color32::from(egui::ecolor::Hsva::new(i as f32 / 6., 0.55, 0.85, 1.));
                p.circle_filled(c + Vec2::angled(a) * 7., 4., color);
            }
            p.circle_filled(c, 3., Color32::from_rgb(242, 204, 92));
        }
        8 => {
            for i in 0..6 {
                let radius = 11. - i as f32 * 1.5;
                let color = Color32::from(egui::ecolor::Hsva::new(i as f32 / 6., 0.55, 0.85, 1.));
                let points = (0..24)
                    .map(|j| {
                        let a = std::f32::consts::PI + j as f32 / 23. * std::f32::consts::PI;
                        c + vec2(a.cos() * radius, a.sin() * radius + 5.)
                    })
                    .collect();
                p.add(egui::Shape::line(points, Stroke::new(1.3_f32, color)));
            }
        }
        128 => {
            for row in 0..3 {
                for col in 0..3 {
                    let hue = (row * 3 + col) as f32 / 9.;
                    let fill = Color32::from(egui::ecolor::Hsva::new(hue, 0.65, 0.85, 1.));
                    p.rect_filled(
                        Rect::from_center_size(
                            c + vec2((col - 1) as f32 * 8., (row - 1) as f32 * 8.),
                            vec2(5., 5.),
                        ),
                        1.,
                        fill,
                    );
                }
            }
            if mode == 128 {
                line(vec2(-9., 10.), vec2(10., -9.));
            }
        }
        7 => {
            for x in [-7., 0., 7.] {
                line(vec2(x, -3.), vec2(x, 3.));
            }
            p.circle_stroke(c, 10., stroke);
        }
        9 | 24 => {
            for i in 0..5 {
                let x = (i - 2) as f32 * 4.;
                let h = if mode == 9 {
                    10. - x.abs() * 0.8
                } else {
                    3. + x.abs() * 0.8
                };
                line(vec2(x, 9.), vec2(x, 9. - h * 2.));
            }
        }
        10 => {
            for x in [-7., 0., 7.] {
                line(vec2(x, -10.), vec2(x, 10.));
            }
            line(vec2(-10., -3.), vec2(10., -3.));
            line(vec2(-10., 4.), vec2(10., 4.));
        }
        12 | 17 => {
            for y in if mode == 12 { vec![0.] } else { vec![-4., 4.] } {
                line(vec2(-11., 7. + y), vec2(-3., -6. + y));
                line(vec2(-3., -6. + y), vec2(3., 3. + y));
                line(vec2(3., 3. + y), vec2(7., -3. + y));
                line(vec2(7., -3. + y), vec2(11., 7. + y));
            }
        }
        14 | 15 => {
            for radius in [3., 7., 11.] {
                p.circle_stroke(c, radius, stroke);
            }
            if mode == 14 {
                p.circle_filled(c + vec2(8., 0.), 2., color);
            }
        }
        18 => {
            for i in 0..4 {
                let x = (i as f32 - 1.5) * 5.;
                line(vec2(x + 3., -9.), vec2(x - 3., 9.));
            }
        }
        19 => {
            line(vec2(-10., -4.), vec2(10., -4.));
            line(vec2(10., 4.), vec2(-10., 4.));
            line(vec2(6., -8.), vec2(10., -4.));
            line(vec2(-6., 8.), vec2(-10., 4.));
        }
        23 => {
            for i in 0..4 {
                let v = Vec2::angled(i as f32 * std::f32::consts::FRAC_PI_2);
                line(Vec2::ZERO, v * 10.);
                line(v * 10., v * 8. + v.rot90() * 6.);
            }
        }
        25 => {
            for offset in [-4., 4.] {
                p.rect_stroke(
                    r.shrink(6.).translate(vec2(offset, offset)),
                    3.,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
        _ => {
            for y in [-6., 0., 6.] {
                let points = (0..18)
                    .map(|i| {
                        let x = i as f32 / 17. * 22. - 11.;
                        c + vec2(x, y + (x * 0.35).sin() * 2.5)
                    })
                    .collect();
                p.add(egui::Shape::line(points, stroke));
            }
        }
    }
}

fn text(
    painter: &egui::Painter,
    position: Pos2,
    text: &str,
    size: f32,
    color: Color32,
    align: Align2,
) {
    painter.text(
        position,
        align,
        tr(text),
        FontId::proportional(size.max(13.)),
        color,
    );
}

fn button(label: &str, selected: bool) -> egui::Button<'_> {
    egui::Button::new(RichText::new(tr(label)).size(14.).color(if selected {
        purple()
    } else {
        ink()
    }))
    .fill(if selected { lilac() } else { surface() })
    .stroke(Stroke::new(
        1.0_f32,
        if selected {
            Color32::from_rgb(211, 191, 252)
        } else {
            line()
        },
    ))
    .corner_radius(4.)
    .min_size(vec2(0., 36.))
    .truncate()
}

// One 24 px icon grid, a fixed 18 px visual width and a common stroke weight.
fn sidebar_icon(p: &egui::Painter, r: Rect, kind: u8, color: Color32) {
    let c = r.center();
    let stroke = Stroke::new(1.35_f32, color);
    let at = |x, y| c + vec2(x, y);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([at(a.0, a.1), at(b.0, b.1)], stroke);
    };
    match kind {
        0 => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(18., 12.)),
                2.,
                stroke,
                StrokeKind::Middle,
            );
            for y in [-2.5, 1.] {
                for x in [-5.5, -1.8, 1.8, 5.5] {
                    p.circle_filled(at(x, y), 0.65, color);
                }
            }
            line((-4.5, 3.5), (4.5, 3.5));
        }
        1 => {
            p.circle_stroke(c, 4.2, stroke);
            for i in 0..8 {
                let d = Vec2::angled(i as f32 * std::f32::consts::TAU / 8.);
                p.line_segment([c + d * 6.5, c + d * 9.], stroke);
            }
        }
        5 => {
            line((-7., -6.), (9., -6.));
            line((9., -6.), (9., 5.));
            line((9., 5.), (-4., 5.));
            line((-4., 5.), (-0.5, 1.5));
            line((-4., 5.), (-0.5, 8.5));
            p.circle_filled(at(-7., -6.), 2., color);
        }
        7 => {
            for x in [-9., 1.] {
                p.rect_stroke(
                    Rect::from_min_size(c + vec2(x, -6.), vec2(8., 12.)),
                    2.,
                    stroke,
                    StrokeKind::Middle,
                );
            }
            line((-3., 0.), (3., 0.));
        }
        6 => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(11., 11.)),
                2.,
                stroke,
                StrokeKind::Middle,
            );
            for a in [-3., 3.] {
                line((-9., a), (-5.5, a));
                line((5.5, a), (9., a));
                line((a, -9.), (a, -5.5));
                line((a, 5.5), (a, 9.));
            }
        }
        _ => {
            for (y, x) in [(-5., -3.), (0., 4.), (5., -1.)] {
                line((-9., y), (9., y));
                p.circle_filled(at(x, y), 2., color);
            }
        }
    }
}

fn icon(p: &egui::Painter, center: Pos2, kind: u8, color: Color32, scale: f32) {
    let stroke = Stroke::new(1.35_f32 * scale, color);
    let point = |x, y| center + vec2(x, y) * scale;
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([point(a.0, a.1), point(b.0, b.1)], stroke);
    };
    match kind {
        0 => {
            p.rect_stroke(
                Rect::from_center_size(center, vec2(20., 14.) * scale),
                2.,
                stroke,
                StrokeKind::Inside,
            );
            for y in [-3., 1.] {
                for x in [-6., -2., 2., 6.] {
                    p.circle_filled(point(x, y), 0.8 * scale, color);
                }
            }
            line((-5., 4.), (5., 4.));
        }
        1 => {
            p.circle_stroke(center, 4. * scale, stroke);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.;
                let dir = vec2(angle.cos(), angle.sin());
                p.line_segment(
                    [center + dir * 6. * scale, center + dir * 9. * scale],
                    stroke,
                );
            }
        }
        2 => {
            p.rect_stroke(
                Rect::from_center_size(center, vec2(17., 17.) * scale),
                3.,
                stroke,
                StrokeKind::Inside,
            );
            line((-4., 0.), (-1., 3.));
            line((-1., 3.), (5., -4.));
        }
        3 => {
            for y in [-5., 4.] {
                for x in [-5., 4.] {
                    p.rect_stroke(
                        Rect::from_min_size(point(x, y), vec2(6., 6.) * scale),
                        1.5,
                        stroke,
                        StrokeKind::Inside,
                    );
                }
            }
        }
        6 => {
            p.rect_stroke(
                Rect::from_center_size(center, vec2(10., 10.) * scale),
                2. * scale,
                stroke,
                StrokeKind::Inside,
            );
            for offset in [-3., 3.] {
                line((-8., offset), (-5., offset));
                line((5., offset), (8., offset));
                line((offset, -8.), (offset, -5.));
                line((offset, 5.), (offset, 8.));
            }
        }
        5 => {
            line((-8., -6.), (7., -6.));
            line((7., -6.), (7., 5.));
            line((7., 5.), (-5., 5.));
            line((-5., 5.), (-2., 2.));
            line((-5., 5.), (-2., 8.));
            p.circle_filled(point(-8., -6.), 2. * scale, color);
        }
        _ => {
            for (y, x) in [(-5., -3.), (0., 4.), (5., -1.)] {
                line((-8., y), (8., y));
                p.circle_filled(point(x, y), 2. * scale, color);
            }
        }
    }
}

fn usage_for_key(key: egui::Key) -> Option<u8> {
    use egui::Key;
    Some(match key {
        Key::A => 4,
        Key::B => 5,
        Key::C => 6,
        Key::D => 7,
        Key::E => 8,
        Key::F => 9,
        Key::G => 10,
        Key::H => 11,
        Key::I => 12,
        Key::J => 13,
        Key::K => 14,
        Key::L => 15,
        Key::M => 16,
        Key::N => 17,
        Key::O => 18,
        Key::P => 19,
        Key::Q => 20,
        Key::R => 21,
        Key::S => 22,
        Key::T => 23,
        Key::U => 24,
        Key::V => 25,
        Key::W => 26,
        Key::X => 27,
        Key::Y => 28,
        Key::Z => 29,
        Key::Num1 => 30,
        Key::Num2 => 31,
        Key::Num3 => 32,
        Key::Num4 => 33,
        Key::Num5 => 34,
        Key::Num6 => 35,
        Key::Num7 => 36,
        Key::Num8 => 37,
        Key::Num9 => 38,
        Key::Num0 => 39,
        Key::Enter => 40,
        Key::Escape => 41,
        Key::Backspace => 42,
        Key::Tab => 43,
        Key::Space => 44,
        Key::Minus => 45,
        Key::Equals => 46,
        Key::OpenBracket => 47,
        Key::CloseBracket => 48,
        Key::Backslash => 49,
        Key::Semicolon => 51,
        Key::Quote => 52,
        Key::Backtick => 53,
        Key::Comma => 54,
        Key::Period => 55,
        Key::Slash => 56,
        Key::F1 => 58,
        Key::F2 => 59,
        Key::F3 => 60,
        Key::F4 => 61,
        Key::F5 => 62,
        Key::F6 => 63,
        Key::F7 => 64,
        Key::F8 => 65,
        Key::F9 => 66,
        Key::F10 => 67,
        Key::F11 => 68,
        Key::F12 => 69,
        Key::Insert => 73,
        Key::Home => 74,
        Key::PageUp => 75,
        Key::Delete => 76,
        Key::End => 77,
        Key::PageDown => 78,
        Key::ArrowRight => 79,
        Key::ArrowLeft => 80,
        Key::ArrowDown => 81,
        Key::ArrowUp => 82,
        _ => return None,
    })
}

/// Shared Options-style dropdown: stable trigger, full-width rows and an explicit checkmark.
struct OptionsCombo {
    id: egui::Id,
    label: String,
    width: f32,
}
fn combo_box(id: impl std::hash::Hash) -> OptionsCombo {
    OptionsCombo {
        id: egui::Id::new(id),
        label: String::new(),
        width: 180.,
    }
}
impl OptionsCombo {
    fn selected_text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }
    fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    fn show_ui<R>(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui) -> R) {
        let id = ui.make_persistent_id(self.id);
        let popup_id = id.with("menu");
        let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        let (rect, _) = ui.allocate_exact_size(vec2(self.width.max(64.), 36.), Sense::hover());
        let response = ui.interact(rect, id, Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), &self.label)
        });
        ui.painter().rect(
            rect,
            6.,
            if response.hovered() { bg() } else { surface() },
            Stroke::new(
                1.0_f32,
                if open || response.has_focus() {
                    purple().gamma_multiply(0.55)
                } else {
                    line()
                },
            ),
            StrokeKind::Inside,
        );
        let painter = ui.painter().with_clip_rect(Rect::from_min_max(
            rect.min + vec2(12., 0.),
            rect.max - vec2(32., 0.),
        ));
        text(
            &painter,
            rect.left_center() + vec2(12., 0.),
            &self.label,
            14.,
            ink(),
            Align2::LEFT_CENTER,
        );
        let c = rect.right_center() - vec2(18., 0.);
        let d = if open { -1. } else { 1. };
        ui.painter().add(egui::Shape::line(
            vec![
                c + vec2(-4.5, -2. * d),
                c + vec2(0., 2.5 * d),
                c + vec2(4.5, -2. * d),
            ],
            Stroke::new(1.5_f32, muted()),
        ));
        let width = self
            .width
            .max(220.)
            .min(ui.ctx().content_rect().width() - 32.);
        egui::Popup::from_toggle_button_response(&response)
            .id(popup_id)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .width(width)
            .frame(
                egui::Frame::popup(ui.style())
                    .fill(surface())
                    .stroke(Stroke::new(1.0_f32, line()))
                    .corner_radius(8.)
                    .inner_margin(6),
            )
            .show(|ui| {
                ui.set_width(width - 12.);
                ui.spacing_mut().item_spacing.y = 2.;
                egui::ScrollArea::vertical()
                    .id_salt(popup_id.with("scroll"))
                    .max_height(304.)
                    .auto_shrink([false, true])
                    .show(ui, content);
            });
    }
}
fn dropdown_option(ui: &mut egui::Ui, selected: bool, label: impl Into<String>) -> egui::Response {
    let label = label.into();
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.), Sense::click());
    if selected || response.hovered() {
        ui.painter()
            .rect_filled(rect, 5., if selected { lilac() } else { bg() });
    }
    text(
        ui.painter(),
        rect.left_center() + vec2(12., 0.),
        &label,
        14.,
        if selected { purple() } else { ink() },
        Align2::LEFT_CENTER,
    );
    if selected {
        let c = rect.right_center() - vec2(15., 0.);
        ui.painter().add(egui::Shape::line(
            vec![c + vec2(-4., 0.), c + vec2(-1., 3.), c + vec2(5., -4.)],
            Stroke::new(1.6_f32, purple()),
        ));
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            &label,
        )
    });
    if response.clicked() {
        ui.close();
    }
    response
}
fn dropdown_value<T: PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    choice: T,
    label: impl Into<String>,
) -> egui::Response {
    let mut response = dropdown_option(ui, *value == choice, label);
    if response.clicked() && *value != choice {
        *value = choice;
        response.mark_changed();
    }
    response
}
