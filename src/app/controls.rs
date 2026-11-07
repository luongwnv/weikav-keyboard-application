use super::*;
use crate::features::{KeyboardSettings, Macro, MacroAction};
use std::collections::BTreeMap;

impl NutApp {
    pub(super) fn macros(&self) -> BTreeMap<u8, Macro> {
        let mut macros = self
            .snapshot
            .as_ref()
            .map(|s| s.macros.clone())
            .unwrap_or_default();
        for (id, m) in &self.profile().macros {
            if m.actions.is_empty() {
                macros.remove(id);
            } else {
                macros.insert(*id, m.clone());
            }
        }
        macros
    }

    pub(super) fn macro_events(&mut self, ctx: &egui::Context) {
        if self.tab != Tab::Macros {
            if self.recording {
                self.stop_recording();
            }
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        let toggle = events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::F8,
                    pressed: true,
                    repeat: false,
                    ..
                }
            )
        });
        if toggle {
            if self.recording {
                self.stop_recording();
                return;
            }
            self.macro_undo = Some((
                self.macro_id,
                self.macros().get(&self.macro_id).cloned().unwrap_or(Macro {
                    name: format!("Macro {}", self.macro_id + 1),
                    actions: Vec::new(),
                }),
            ));
            self.recording = true;
            self.last_record = Instant::now();
            self.record_modifiers = 0;
        }
        if !self.recording {
            return;
        }
        let mut m = self
            .profile()
            .macros
            .get(&self.macro_id)
            .cloned()
            .or_else(|| self.macros().get(&self.macro_id).cloned())
            .unwrap_or(Macro {
                name: format!("Macro {}", self.macro_id + 1),
                actions: Vec::new(),
            });
        let count = m.actions.len();
        let modifiers = ctx.input(|i| i.modifiers);
        let next = u8::from(modifiers.ctrl)
            | u8::from(modifiers.shift) << 1
            | u8::from(modifiers.alt) << 2
            | u8::from(modifiers.mac_cmd || (modifiers.command && !modifiers.ctrl)) << 3;
        for bit in 0..4 {
            let mask = 1 << bit;
            if self.record_modifiers & mask != next & mask {
                m.actions.push(MacroAction {
                    delay: if self.record_delays {
                        self.last_record.elapsed().as_millis().min(65535) as u16
                    } else {
                        0
                    },
                    key: 224 + bit,
                    status: if next & mask != 0 { 0x90 } else { 0x10 },
                });
                self.last_record = Instant::now();
            }
        }
        self.record_modifiers = next;
        for e in events {
            if let egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat: false,
                ..
            } = e
            {
                if key == egui::Key::F8 {
                    continue;
                }
                if let Some(usage) = usage_for_key(physical_key.unwrap_or(key)) {
                    m.actions.push(MacroAction {
                        delay: if self.record_delays {
                            self.last_record.elapsed().as_millis().min(65535) as u16
                        } else {
                            0
                        },
                        key: usage,
                        status: if pressed { 0x90 } else { 0x10 },
                    });
                    self.last_record = Instant::now();
                }
            }
        }
        if m.actions.len() != count {
            let mut complete = m.clone();
            complete.release_held();
            let mut all = self.macros();
            all.insert(self.macro_id, complete);
            if crate::features::macro_bytes(&all).is_err() {
                self.stop_recording();
                self.tell("Bộ nhớ macro đã đầy", true);
            } else {
                let id = self.macro_id;
                self.profile_mut().macros.insert(id, m);
                self.changed();
            }
        }
    }

    pub(super) fn stop_recording(&mut self) {
        if !self.recording {
            return;
        }
        self.recording = false;
        if let Some(mut m) = self.macros().get(&self.macro_id).cloned() {
            let before = m.actions.len();
            m.release_held();
            if m.actions.len() != before {
                let id = self.macro_id;
                self.profile_mut().macros.insert(id, m);
                self.changed();
            }
        }
    }

    pub(super) fn binding_reset(&mut self, ui: &mut egui::Ui, id: u8) {
        ui.add_space(12.);
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new(tr("Khôi phục mặc định")).size(13.))
                        .frame(false),
                )
                .clicked()
            {
                self.assign(DEFAULT);
            }
            if self.profile().patches(self.function).contains_key(&id)
                && ui
                    .add(egui::Button::new(RichText::new(tr("Hoàn tác")).size(13.)).frame(false))
                    .clicked()
            {
                let function = self.function;
                self.profile_mut().patches_mut(function).remove(&id);
                self.changed();
            }
        });
    }

    pub(super) fn macro_bindings(&mut self, ui: &mut egui::Ui) {
        let macros = self.macros();
        let before = (self.macro_mode, self.macro_repeat);
        let current = self.selected.map(|id| self.binding(id)).unwrap_or(DEFAULT);
        egui::Grid::new("macro_playback")
            .num_columns(2)
            .spacing([12., 6.])
            .show(ui, |ui| {
                ui.label(RichText::new(tr("Phát")).size(13.).color(muted()));
                combo_box("macro_mode")
                    .selected_text(tr(match self.macro_mode {
                        1 => "Lặp số lần",
                        2 => "Nhấn lại để dừng",
                        _ => "Một lần",
                    }))
                    .width((ui.available_width() - 4.).max(140.))
                    .show_ui(ui, |ui| {
                        dropdown_value(ui, &mut self.macro_mode, 0, tr("Một lần"));
                        dropdown_value(ui, &mut self.macro_mode, 1, tr("Lặp số lần"));
                        dropdown_value(ui, &mut self.macro_mode, 2, tr("Nhấn lại để dừng"));
                    });
                ui.end_row();
                if self.macro_mode == 1 {
                    ui.label(RichText::new(tr("Lần")).size(13.).color(muted()));
                    ui.add_sized(
                        [80., 32.],
                        egui::DragValue::new(&mut self.macro_repeat)
                            .range(1..=255)
                            .speed(1.),
                    );
                    ui.end_row();
                }
            });
        if before != (self.macro_mode, self.macro_repeat) && current[0] == 6 {
            self.assign([
                6,
                current[1],
                self.macro_mode,
                if self.macro_mode == 1 {
                    self.macro_repeat
                } else {
                    0
                },
            ]);
        }
        ui.add_space(10.);
        for (id, m) in macros {
            if super::macro_editor::menu_row(ui, &m.name, true, current[0] == 6 && current[1] == id)
                .clicked()
            {
                self.assign([
                    6,
                    id,
                    self.macro_mode,
                    if self.macro_mode == 1 {
                        self.macro_repeat
                    } else {
                        0
                    },
                ]);
            }
        }
        ui.add_space(4.);
        if ui
            .add(
                egui::Button::new(RichText::new(tr("Quản lý macro")).size(13.).color(purple()))
                    .frame(false),
            )
            .clicked()
        {
            self.tab = Tab::Macros;
            self.selected = None;
        }
    }

    pub(super) fn advanced_inspector(&mut self, ui: &mut egui::Ui) {
        let old_kind = self.advanced_kind;
        combo_box("advanced_type")
            .selected_text(tr([
                "MT - Chạm / giữ",
                "TGL - Bật / tắt",
                "SOCD - Hai phím đối hướng",
                "CB - Tổ hợp phím",
            ][self.advanced_kind]))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, name) in [
                    "MT - Chạm / giữ",
                    "TGL - Bật / tắt",
                    "SOCD - Hai phím đối hướng",
                    "CB - Tổ hợp phím",
                ]
                .iter()
                .enumerate()
                {
                    dropdown_value(ui, &mut self.advanced_kind, i, tr(name));
                }
            });
        if old_kind != self.advanced_kind && self.advanced_kind == 3 {
            self.advanced_keys = [224, 0, 6];
        }
        ui.add_space(16.);
        match self.advanced_kind {
            0 => {
                key_choice(ui, "Chạm", &mut self.advanced_keys[0], false);
                key_choice(ui, "Giữ", &mut self.advanced_keys[1], false);
                options_slider(
                    ui,
                    egui::Slider::new(&mut self.advanced_delay, 10..=1000)
                        .step_by(10.)
                        .suffix(" ms"),
                );
            }
            1 => key_choice(ui, "Phím", &mut self.advanced_keys[0], false),
            2 => {
                key_choice(ui, "Phím đối hướng", &mut self.advanced_keys[2], false);
                let names = [
                    "Ưu tiên phím 1",
                    "Ưu tiên phím 2",
                    "Ưu tiên phím cuối",
                    "Trung hoà",
                ];
                combo_box("socd_mode")
                    .selected_text(tr(names[(self.advanced_mode - 1) as usize]))
                    .show_ui(ui, |ui| {
                        for (i, name) in names.iter().enumerate() {
                            dropdown_value(ui, &mut self.advanced_mode, i as u8 + 1, tr(name));
                        }
                    });
            }
            _ => {
                key_choice(ui, "Bổ trợ 1", &mut self.advanced_keys[0], true);
                key_choice(ui, "Bổ trợ 2", &mut self.advanced_keys[1], true);
                key_choice(ui, "Phím", &mut self.advanced_keys[2], false);
            }
        }
        ui.add_space(18.);
        if ui
            .add_sized([ui.available_width(), 36.], button("Gán chức năng", true))
            .clicked()
        {
            let [a, b, c] = self.advanced_keys;
            match self.advanced_kind {
                0 => self.assign([9, a, b, (self.advanced_delay / 10) as u8]),
                1 => self.assign([10, a, 0, 0]),
                2 => {
                    if let Some(id) = self.selected {
                        let first = self
                            .areas
                            .iter()
                            .find(|a| a.id == id)
                            .map(|a| a.key.key_code as u8);
                        let second = self
                            .areas
                            .iter()
                            .find(|a| a.key.key_code == c as u16 && a.id != 85)
                            .map(|a| a.id);
                        if let (Some(first), Some(second)) = (first, second) {
                            if id != second && !self.locked(second) {
                                let binding = [11, self.advanced_mode, first, c];
                                self.assign(binding);
                                let function = self.function;
                                self.profile_mut()
                                    .patches_mut(function)
                                    .insert(second, binding);
                                self.changed();
                            } else {
                                self.tell("Chọn hai phím khác nhau và có thể chỉnh sửa", true);
                            }
                        }
                    }
                }
                _ => self.assign([
                    7,
                    if a >= 224 { a } else { 0 },
                    if b >= 224 { b } else { 0 },
                    c,
                ]),
            }
        }
    }

    pub(super) fn keyboard_settings(&mut self, ui: &mut egui::Ui) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let mut settings = self
            .profile()
            .settings
            .clone()
            .unwrap_or_else(|| KeyboardSettings::from_raw(&snapshot.settings));
        let before = settings.clone();
        ui.label(RichText::new(tr("Kết nối và phản hồi")).size(16.).strong());
        ui.add_space(8.);
        for (id, label, value, choices) in [
            (
                "system",
                "Hệ điều hành",
                &mut settings.system,
                vec![(0, "Windows"), (1, "macOS"), (2, "Android"), (3, "iOS")],
            ),
            (
                "report",
                "Polling rate",
                &mut settings.report_rate,
                vec![(3, "1.000 Hz"), (5, "4.000 Hz"), (6, "8.000 Hz")],
            ),
            (
                "sleep",
                "Ngủ sau",
                &mut settings.sleep,
                vec![
                    (0, "Không ngủ"),
                    (1, "1 phút"),
                    (3, "3 phút"),
                    (5, "5 phút"),
                    (10, "10 phút"),
                    (30, "30 phút"),
                ],
            ),
        ] {
            let name = choices
                .iter()
                .find(|(v, _)| v == value)
                .map(|(_, n)| *n)
                .unwrap_or("Theo thiết bị");
            super::settings::device_control_row(ui, tr(label), |ui| {
                combo_box(id)
                    .selected_text(tr(name))
                    .width(180.)
                    .show_ui(ui, |ui| {
                        for (v, n) in choices {
                            dropdown_value(ui, value, v, tr(n));
                        }
                    });
            });
        }
        super::settings::device_control_row(ui, tr("Chống dội phím"), |ui| {
            ui.spacing_mut().slider_width = 96.;
            options_slider(
                ui,
                egui::Slider::new(&mut settings.debounce, 1..=5).suffix(tr(" mức")),
            );
        });
        ui.add_space(8.);
        ui.separator();
        ui.add_space(12.);
        ui.label(RichText::new(tr("Chức năng phím")).size(16.).strong());
        ui.add_space(6.);
        for (name, value) in [
            ("Khoá phím Win", &mut settings.game_mode),
            ("Đổi hàng F1–F12", &mut settings.fn_switch),
            ("N-key rollover", &mut settings.nkro),
        ] {
            super::settings::device_switch(ui, tr(name), value);
        }

        if settings != before {
            self.profile_mut().settings = Some(settings);
            self.changed();
        }
    }
}

fn key_choice(ui: &mut egui::Ui, label: &str, value: &mut u8, modifiers: bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr(label)).size(14.));
        combo_box(label)
            .selected_text(if *value == 0 {
                tr("Không").into()
            } else {
                model::key_label(*value)
            })
            .width(150.)
            .show_ui(ui, |ui| {
                if modifiers {
                    dropdown_value(ui, value, 0, tr("Không"));
                }
                let keys: Vec<u8> = if modifiers {
                    (224..=231).collect()
                } else {
                    (4..=115).chain(224..=231).collect()
                };
                for key in keys {
                    dropdown_value(ui, value, key, model::key_label(key));
                }
            });
    });
}
