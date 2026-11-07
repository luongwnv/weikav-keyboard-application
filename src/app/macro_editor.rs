use super::*;
use crate::features::{Macro, MacroAction};
impl NutApp {
    pub(super) fn macro_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        let body = page_body(ui, r, "Macro");
        let width = body.width().min(820.);
        let content = Rect::from_min_max(
            pos2(body.center().x - width / 2., body.top()),
            pos2(body.center().x + width / 2., body.bottom()),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content));
        let ui = &mut child;
        let mut all = self.macros();
        all.extend(self.profile().macros.clone());
        let selected = self.macro_id;
        let mut m = all.get(&selected).cloned().unwrap_or(Macro {
            name: format!("Macro {}", selected + 1),
            actions: Vec::new(),
        });
        let before = m.clone();
        let mut start = false;
        let mut stop = false;
        let mut restore = false;
        let mut clear = false;
        let mut move_action = None;
        let mut remove_action = None;
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!self.recording, |ui| {
                combo_box("macro_slot")
                    .selected_text(format!("M{}", selected + 1))
                    .width(72.)
                    .show_ui(ui, |ui| {
                        for id in 0..=all.keys().copied().max().unwrap_or(0).max(7) {
                            let name = all
                                .get(&id)
                                .map(|m| m.name.clone())
                                .unwrap_or(format!("Macro {}", id + 1));
                            if dropdown_option(ui, id == selected, format!("M{} - {name}", id + 1))
                                .clicked()
                            {
                                self.macro_id = id;
                                self.macro_action = None;
                                self.macro_add_open = false;
                                ui.close();
                            }
                        }
                    });
                ui.add(
                    egui::TextEdit::singleline(&mut m.name)
                        .char_limit(60)
                        .font(FontId::proportional(18.))
                        .frame(false)
                        .desired_width((width - 190.).max(150.))
                        .margin(vec2(8., 8.)),
                );
                if macro_header_tool(ui, false)
                    .on_hover_text(tr("Macro mới"))
                    .clicked()
                {
                    let id = all.keys().copied().max().unwrap_or(7).max(7) + 1;
                    if id < 100 {
                        self.profile_mut().macros.insert(
                            id,
                            Macro {
                                name: format!("Macro {}", id + 1),
                                actions: Vec::new(),
                            },
                        );
                        self.macro_id = id;
                        self.macro_action = None;
                        self.changed();
                    }
                }
                let more = macro_header_tool(ui, true).on_hover_text(tr("Tuỳ chọn"));
                egui::Popup::from_toggle_button_response(&more)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .width(232.)
                    .frame(
                        egui::Frame::popup(ui.style())
                            .fill(surface())
                            .stroke(Stroke::new(1.0_f32, line()))
                            .corner_radius(6.)
                            .inner_margin(6),
                    )
                    .show(|ui| {
                        ui.set_width(220.);
                        ui.spacing_mut().item_spacing.y = 2.;
                        if menu_row(ui, tr("Ghi độ trễ"), true, self.record_delays).clicked() {
                            self.record_delays = !self.record_delays;
                        }
                        if menu_row(ui, tr("Bảng chi tiết"), true, self.macro_table).clicked() {
                            self.macro_table = !self.macro_table;
                        }
                        ui.separator();
                        if menu_row(
                            ui,
                            tr("Hoàn tác"),
                            self.macro_undo
                                .as_ref()
                                .is_some_and(|(id, _)| *id == selected),
                            false,
                        )
                        .clicked()
                        {
                            restore = true;
                            ui.close();
                        }
                        if menu_row(ui, tr("Nhân bản"), !m.actions.is_empty(), false).clicked() {
                            if let Some(id) =
                                (0..100).find(|id| *id != selected && !all.contains_key(id))
                            {
                                let mut copy = m.clone();
                                copy.name = format!(
                                    "{} - {}",
                                    m.name.chars().take(50).collect::<String>(),
                                    tr("bản sao")
                                );
                                let mut check = self.macros();
                                check.insert(id, copy.clone());
                                if crate::features::macro_bytes(&check).is_ok() {
                                    self.profile_mut().macros.insert(id, copy);
                                    self.macro_id = id;
                                    self.changed();
                                } else {
                                    self.tell("Bộ nhớ macro đã đầy", true);
                                }
                            }
                            ui.close();
                        }
                        if menu_row(ui, tr("Xoá thao tác"), !m.actions.is_empty(), false).clicked()
                        {
                            clear = true;
                            ui.close();
                        }
                        ui.separator();
                        let used: usize = self
                            .macros()
                            .values()
                            .filter(|m| !m.actions.is_empty())
                            .map(|m| 4 + m.actions.len() * 4)
                            .sum();
                        ui.add_space(4.);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(tr("Bộ nhớ")).size(12.).color(muted()));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(format!("{}%", used * 100 / 512))
                                            .size(12.)
                                            .color(muted()),
                                    );
                                },
                            );
                        });
                        let (bar, _) =
                            ui.allocate_exact_size(vec2(ui.available_width(), 3.), Sense::hover());
                        ui.painter().rect_filled(bar, 2., line());
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                bar.min,
                                vec2(bar.width() * (used as f32 / 512.).min(1.), 3.),
                            ),
                            2.,
                            purple(),
                        );
                        ui.add_space(4.);
                    });
            });
        });
        ui.add_space(28.);
        ui.label(RichText::new(tr("Khi nhấn")).size(14.).strong());
        ui.add_space(8.);
        let (trigger, response) = ui.allocate_exact_size(vec2(width, 54.), Sense::click());
        ui.painter().rect(
            trigger,
            5.,
            surface(),
            Stroke::new(1.0_f32, line()),
            StrokeKind::Inside,
        );
        icon(
            ui.painter(),
            trigger.left_center() + vec2(26., 0.),
            0,
            ink(),
            0.9,
        );
        let triggers: Vec<_> = self
            .areas
            .iter()
            .filter(|a| {
                self.profile()
                    .base
                    .get(&a.id)
                    .copied()
                    .or_else(|| self.snapshot.as_ref().map(|s| s.binding(false, a.id)))
                    .is_some_and(|b| b[0] == 6 && b[1] == selected)
            })
            .map(|a| a.key.name.clone())
            .collect();
        let label = if triggers.is_empty() {
            tr("Chọn phím kích hoạt").to_owned()
        } else {
            triggers.join(" - ")
        };
        text(
            ui.painter(),
            trigger.left_center() + vec2(54., 0.),
            &label,
            15.,
            ink(),
            Align2::LEFT_CENTER,
        );
        text(
            ui.painter(),
            trigger.right_center() - vec2(24., 0.),
            "›",
            22.,
            muted(),
            Align2::CENTER_CENTER,
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Chọn phím kích hoạt"))
        });
        if response.clicked() && !self.recording {
            self.tab = Tab::Keys;
            self.category = 3;
            self.selected = None;
        }
        ui.add_space(28.);
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("Sau đó")).size(14.).strong());
            ui.add_enabled_ui(!self.recording, |ui| {
                let selected = self.macro_action.filter(|index| *index < m.actions.len());
                if action_tool(ui, 0, "Di chuyển lên", selected.is_some_and(|i| i > 0)).clicked()
                {
                    let i = selected.unwrap();
                    move_action = Some((i, i - 1));
                }
                if action_tool(
                    ui,
                    1,
                    "Di chuyển xuống",
                    selected.is_some_and(|i| i + 1 < m.actions.len()),
                )
                .clicked()
                {
                    let i = selected.unwrap();
                    move_action = Some((i, i + 2));
                }
                if action_tool(ui, 2, "Xoá thao tác", selected.is_some()).clicked() {
                    remove_action = selected;
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if self.recording {
                    tr("Dừng ghi")
                } else {
                    tr("Ghi phím")
                };
                if ui
                    .add_sized(
                        [124., 36.],
                        egui::Button::new(
                            RichText::new(format!(
                                "{}  {label}",
                                if self.recording { "■" } else { "●" }
                            ))
                            .size(14.)
                            .color(Color32::WHITE),
                        )
                        .fill(brand())
                        .stroke(Stroke::NONE)
                        .corner_radius(4.),
                    )
                    .on_hover_text("F8")
                    .clicked()
                {
                    if self.recording {
                        stop = true;
                    } else {
                        start = true;
                    }
                }
            });
        });
        ui.add_space(12.);
        egui::ScrollArea::vertical()
            .id_salt("macro_workflow")
            .max_height((content.bottom() - ui.cursor().top() - 12.).max(150.))
            .show(ui, |ui| {
                let width = ui.available_width();
                ui.set_width(width);
                if self.macro_table && !m.actions.is_empty() {
                    let (header, _) = ui.allocate_exact_size(vec2(width, 24.), Sense::hover());
                    for (offset, label) in [
                        (44., tr("Phím")),
                        (110., tr("Thao tác")),
                        (width - 94., "ms"),
                    ] {
                        text(
                            ui.painter(),
                            header.left_center() + vec2(offset, 0.),
                            label,
                            12.,
                            muted(),
                            Align2::LEFT_CENTER,
                        );
                    }
                }
                let (moved, clicked) = animated_action_list(
                    ui,
                    width,
                    selected,
                    &mut m.actions,
                    self.recording,
                    self.macro_action,
                    |ui, width, action, active| self.draw_macro_row(ui, width, action, active),
                );
                if moved.is_some() {
                    move_action = moved;
                }
                if clicked.is_some() {
                    self.macro_action = clicked;
                }

                let expanded = m.actions.is_empty() || self.macro_add_open;
                let (header, response) = ui.allocate_exact_size(vec2(width, 44.), Sense::click());
                ui.painter().rect_filled(header, 5., brand());
                text(
                    ui.painter(),
                    header.left_center() + vec2(20., 0.),
                    &format!("+  {}", tr("Thêm thao tác")),
                    15.,
                    Color32::WHITE,
                    Align2::LEFT_CENTER,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Thêm thao tác"))
                });
                if response.clicked() {
                    self.macro_add_open = !self.macro_add_open;
                }
                if expanded {
                    for (name, kind) in [
                        ("Chạm phím", 0),
                        ("Nhấn phím", 1),
                        ("Thả phím", 2),
                        ("Bấm chuột", 3),
                    ] {
                        let (row, response) =
                            ui.allocate_exact_size(vec2(width, 44.), Sense::click());
                        ui.painter().rect(
                            row,
                            0.,
                            if response.hovered() {
                                lilac()
                            } else {
                                surface()
                            },
                            Stroke::new(1.0_f32, line()),
                            StrokeKind::Inside,
                        );
                        icon(
                            ui.painter(),
                            row.left_center() + vec2(26., 0.),
                            if kind == 3 { 4 } else { 0 },
                            ink(),
                            0.85,
                        );
                        text(
                            ui.painter(),
                            row.left_center() + vec2(54., 0.),
                            name,
                            15.,
                            ink(),
                            Align2::LEFT_CENTER,
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                !self.recording,
                                tr(name),
                            )
                        });
                        if response.clicked() && !self.recording {
                            let index = m.actions.len();
                            let (key, down, up) = if kind == 3 {
                                (1, 0xb0, 0x30)
                            } else {
                                (4, 0x90, 0x10)
                            };
                            if kind != 2 {
                                m.actions.push(MacroAction {
                                    key,
                                    status: down,
                                    delay: 0,
                                });
                            }
                            if kind != 1 {
                                m.actions.push(MacroAction {
                                    key,
                                    status: up,
                                    delay: 50,
                                });
                            }
                            self.macro_action = Some(index);
                            self.macro_add_open = false;
                        }
                    }
                }
            });
        if let Some(index) = remove_action.filter(|i| *i < m.actions.len()) {
            m.actions.remove(index);
            self.macro_action = if m.actions.is_empty() {
                None
            } else {
                Some(index.min(m.actions.len() - 1))
            };
        } else if let Some((from, to)) = move_action {
            self.macro_action = m.move_action(from, to);
        }
        if clear {
            m.actions.clear();
            self.macro_action = None;
        }
        if restore && let Some((_, old)) = self.macro_undo.take() {
            m = old;
            self.macro_action = None;
        }
        if m != before {
            let mut check = self.macros();
            check.insert(selected, m.clone());
            if crate::features::macro_bytes(&check).is_ok() {
                if !restore {
                    self.macro_undo = Some((selected, before.clone()));
                }
                self.profile_mut().macros.insert(selected, m);
                self.changed();
            } else {
                self.tell("Bộ nhớ macro đã đầy", true);
            }
        }
        if start {
            self.macro_undo = Some((selected, before));
            self.recording = true;
            self.last_record = Instant::now();
            self.record_modifiers = 0;
            self.macro_action = None;
        }
        if stop {
            self.stop_recording();
        }
    }
    fn draw_macro_row(
        &self,
        ui: &mut egui::Ui,
        width: f32,
        action: &mut MacroAction,
        active: bool,
    ) -> (Rect, egui::Response) {
        if !self.macro_table {
            return self.draw_macro_action(ui, width, action, active);
        }
        let (row, response) = ui.allocate_exact_size(vec2(width, 52.), Sense::click());
        if ui.layer_id().order == egui::Order::Tooltip {
            ui.painter().rect_filled(
                row.expand(3.).translate(vec2(0., 4.)),
                7.,
                Color32::from_black_alpha(18),
            );
        }
        ui.painter().rect(
            row,
            5.,
            if active { lilac() } else { surface() },
            Stroke::new(1.0_f32, if active { purple() } else { line() }),
            StrokeKind::Inside,
        );
        drag_grip(ui.painter(), row.left_center() + vec2(16., 0.), muted());
        self.paint_macro_keycap(
            ui.painter(),
            row.left_center() + vec2(64., 0.),
            action,
            active,
        );
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(Rect::from_min_size(
                row.min + vec2(110., 8.),
                vec2(136., 36.),
            )),
            |ui| {
                let mut press = action.status & 0x80 != 0;
                ui.add_enabled_ui(!self.recording, |ui| {
                    combo_box("table_action")
                        .width(126.)
                        .selected_text(tr(if press { "Nhấn" } else { "Thả" }))
                        .show_ui(ui, |ui| {
                            dropdown_value(ui, &mut press, true, tr("Nhấn"));
                            dropdown_value(ui, &mut press, false, tr("Thả"));
                        });
                });
                action.status = (action.status & 0x7f) | if press { 0x80 } else { 0 };
            },
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    row.right_top() + vec2(-102., 8.),
                    vec2(88., 36.),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.add_enabled(
                    !self.recording,
                    egui::DragValue::new(&mut action.delay)
                        .range(0..=65535)
                        .suffix(" ms")
                        .speed(1.),
                );
            },
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                true,
                active,
                action_name(action),
            )
        });
        (row, response)
    }
    fn draw_macro_action(
        &self,
        ui: &mut egui::Ui,
        width: f32,
        action: &MacroAction,
        active: bool,
    ) -> (Rect, egui::Response) {
        let (row, response) = ui.allocate_exact_size(vec2(width, 52.), Sense::click());
        let color = if active { purple() } else { ink() };
        if ui.layer_id().order == egui::Order::Tooltip {
            ui.painter().rect_filled(
                row.expand(3.).translate(vec2(0., 4.)),
                7.,
                Color32::from_black_alpha(18),
            );
        }
        ui.painter().rect(
            row,
            5.,
            if active { lilac() } else { surface() },
            Stroke::new(1.0_f32, if active { purple() } else { line() }),
            StrokeKind::Inside,
        );
        self.paint_macro_keycap(
            ui.painter(),
            row.left_center() + vec2(64., 0.),
            action,
            active,
        );
        let name = format!(
            "{}  {}",
            tr(if action.status & 0x80 != 0 {
                "Nhấn"
            } else {
                "Thả"
            }),
            action_name(action)
        );
        text(
            ui.painter(),
            row.left_center() + vec2(110., 0.),
            &name,
            15.,
            color,
            Align2::LEFT_CENTER,
        );
        text(
            ui.painter(),
            row.right_center() - vec2(22., 0.),
            &format!("{} ms", action.delay),
            13.,
            if active { purple() } else { muted() },
            Align2::RIGHT_CENTER,
        );
        drag_grip(
            ui.painter(),
            row.left_center() + vec2(16., 0.),
            if active { purple() } else { muted() },
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, &name)
        });
        (row, response)
    }

    fn paint_macro_keycap(
        &self,
        p: &egui::Painter,
        center: Pos2,
        action: &MacroAction,
        selected: bool,
    ) {
        if action.status & 0x70 == 0x10
            && self.paint_keyboard_keycap(p, center, u16::from(action.key), selected)
        {
            return;
        }
        {
            let r = Rect::from_center_size(center, vec2(48., 30.));
            p.rect(
                r,
                4.,
                surface(),
                Stroke::new(1.0_f32, line()),
                StrokeKind::Inside,
            );
            let name = action_name(action);
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                name,
                FontId::proportional(10.),
                ink(),
            );
        }
    }

    /// Shared native keycap renderer for macro actions and keyboard shortcuts.
    pub(super) fn paint_keyboard_keycap(
        &self,
        p: &egui::Painter,
        center: Pos2,
        usage: u16,
        selected: bool,
    ) -> bool {
        self.paint_keyboard_keycap_at_height(p, center, usage, selected, 32.)
    }
    pub(super) fn paint_keyboard_keycap_at_height(
        &self,
        p: &egui::Painter,
        center: Pos2,
        usage: u16,
        selected: bool,
        height: f32,
    ) -> bool {
        let Some(area) = self.areas.iter().find(|area| area.key.key_code == usage) else {
            return false;
        };
        let [x, y, w, h] = area.rect;
        let scale = (height / h).min(height * 2.25 / w);
        let cap = Rect::from_center_size(center, vec2(w, h) * scale);
        let board = Rect::from_min_size(cap.min - vec2(x, y) * scale, vec2(1000., 400.) * scale);
        let clipped = p.with_clip_rect(cap.expand(0.5));
        preview::paint(
            &clipped,
            board,
            &self.areas,
            self.library.colorway,
            None,
            self.keyboard.id(),
            &Default::default(),
        );
        if selected {
            p.rect_stroke(
                cap.expand(1.),
                3.,
                Stroke::new(1.3_f32, Color32::WHITE),
                StrokeKind::Outside,
            );
        }
        true
    }

    pub(super) fn macro_inspector(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.macro_action else {
            return;
        };
        let id = self.macro_id;
        let mut m = self.macros().get(&id).cloned().unwrap_or(Macro {
            name: format!("Macro {}", id + 1),
            actions: Vec::new(),
        });
        if index >= m.actions.len() {
            self.macro_action = None;
            return;
        }
        let before = m.clone();
        let len = m.actions.len();
        let mut remove = false;
        let mut up = false;
        let mut down = false;
        ui.add_enabled_ui(!self.recording, |ui| {
            ui.label(RichText::new(tr("Phím / chuột")).size(14.));
            action_key(ui, &mut m.actions[index]);
            ui.add_space(24.);
            ui.label(RichText::new(tr("Thao tác")).size(14.));
            let a = &mut m.actions[index];
            let mut press = a.status & 0x80 != 0;
            combo_box("action_state")
                .width(ui.available_width())
                .selected_text(tr(if press { "Nhấn" } else { "Thả" }))
                .show_ui(ui, |ui| {
                    dropdown_value(ui, &mut press, true, tr("Nhấn"));
                    dropdown_value(ui, &mut press, false, tr("Thả"));
                });
            a.status = (a.status & 0x7f) | if press { 0x80 } else { 0 };
            ui.add_space(24.);
            ui.label(RichText::new(tr("Trễ trước thao tác")).size(14.));
            ui.add(
                egui::DragValue::new(&mut a.delay)
                    .range(0..=65535)
                    .speed(10)
                    .suffix(" ms"),
            );
            ui.add_space(28.);
            ui.horizontal(|ui| {
                if ui.add_enabled(index > 0, button("↑", false)).clicked() {
                    up = true;
                }
                if ui
                    .add_enabled(index + 1 < len, button("↓", false))
                    .clicked()
                {
                    down = true;
                }
            });
            ui.add_space(16.);
            if ui
                .add_sized([ui.available_width(), 36.], button("Xoá thao tác", false))
                .clicked()
            {
                remove = true;
            }
        });
        if remove {
            m.actions.remove(index);
            self.macro_action = None;
        }
        if up {
            m.actions.swap(index, index - 1);
            self.macro_action = Some(index - 1);
        }
        if down {
            m.actions.swap(index, index + 1);
            self.macro_action = Some(index + 1);
        }
        if m != before {
            self.macro_undo = Some((id, before));
            self.profile_mut().macros.insert(id, m);
            self.changed();
        }
    }
}

fn action_name(a: &MacroAction) -> String {
    if a.status & 0x70 == 0x30 {
        tr(match a.key {
            1 => "Chuột trái",
            2 => "Chuột phải",
            4 => "Chuột giữa",
            8 => "Quay lại",
            16 => "Tiến tới",
            _ => "Chuột",
        })
        .into()
    } else {
        model::key_label(a.key)
    }
}
fn action_key(ui: &mut egui::Ui, a: &mut MacroAction) {
    let mouse = a.status & 0x70 == 0x30;
    combo_box("action_key")
        .selected_text(action_name(a))
        .width(ui.available_width() - 8.)
        .show_ui(ui, |ui| {
            for key in (4..=0x73).chain(0xe0..=0xe7) {
                if dropdown_option(ui, !mouse && key == a.key, model::key_label(key)).clicked() {
                    a.key = key;
                    a.status = if a.status & 0x80 != 0 { 0x90 } else { 0x10 };
                }
            }
            for (mask, name) in [
                (1, "Chuột trái"),
                (2, "Chuột phải"),
                (4, "Chuột giữa"),
                (8, "Quay lại"),
                (16, "Tiến tới"),
            ] {
                if dropdown_option(ui, mouse && a.key == mask, tr(name)).clicked() {
                    a.key = mask;
                    a.status = if a.status & 0x80 != 0 { 0xb0 } else { 0x30 };
                }
            }
        });
}

/// Compact list rows shared by macro choices and the editor's overflow menu.
pub(super) fn menu_row(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    selected: bool,
) -> egui::Response {
    let (r, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 36.),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, label)
    });
    if selected || (enabled && response.hovered()) {
        ui.painter().rect_filled(r, 4., lilac());
    }
    let color = if !enabled {
        muted().gamma_multiply(0.55)
    } else if selected {
        purple()
    } else {
        ink()
    };
    text(
        ui.painter(),
        r.left_center() + vec2(12., 0.),
        label,
        14.,
        color,
        Align2::LEFT_CENTER,
    );
    if selected {
        let c = r.right_center() - vec2(17., 0.);
        ui.painter().add(egui::Shape::line(
            vec![c + vec2(-4., 0.), c + vec2(-1., 3.), c + vec2(5., -4.)],
            Stroke::new(1.6_f32, purple()),
        ));
    }
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

#[derive(Clone)]
struct MacroDrag {
    slot: u8,
    index: usize,
}
fn drag_grip(p: &egui::Painter, center: Pos2, color: Color32) {
    for x in [-2., 2.] {
        for y in [-4., 0., 4.] {
            p.circle_filled(center + vec2(x, y), 0.8, color);
        }
    }
}
fn action_tool(ui: &mut egui::Ui, kind: u8, name: &str, enabled: bool) -> egui::Response {
    let (r, response) = ui.allocate_exact_size(
        vec2(32., 32.),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, tr(name)));
    ui.painter().rect(
        r,
        4.,
        if enabled && response.hovered() {
            lilac()
        } else {
            surface()
        },
        Stroke::new(1.0_f32, line()),
        StrokeKind::Inside,
    );
    let color = if enabled {
        ink()
    } else {
        muted().gamma_multiply(0.4)
    };
    let c = r.center();
    let stroke = Stroke::new(1.4_f32, color);
    if kind < 2 {
        let sign = if kind == 0 { -1. } else { 1. };
        ui.painter().add(egui::Shape::line(
            vec![
                c + vec2(-4., -2. * sign),
                c + vec2(0., 2. * sign),
                c + vec2(4., -2. * sign),
            ],
            stroke,
        ));
    } else {
        ui.painter().rect_stroke(
            Rect::from_center_size(c + vec2(0., 1.5), vec2(8., 9.)),
            1.,
            stroke,
            StrokeKind::Middle,
        );
        ui.painter()
            .line_segment([c + vec2(-6., -4.), c + vec2(6., -4.)], stroke);
        ui.painter()
            .line_segment([c + vec2(-2., -6.), c + vec2(2., -6.)], stroke);
    }
    response.on_hover_text(tr(name))
}

fn macro_header_tool(ui: &mut egui::Ui, more: bool) -> egui::Response {
    let (r, response) = ui.allocate_exact_size(vec2(36., 36.), Sense::click());
    ui.painter().rect(
        r,
        4.,
        if response.hovered() {
            lilac()
        } else {
            surface()
        },
        Stroke::new(1.0_f32, line()),
        StrokeKind::Inside,
    );
    let c = r.center();
    if more {
        for x in [-5., 0., 5.] {
            ui.painter().circle_filled(c + vec2(x, 0.), 1.5, ink());
        }
    } else {
        let stroke = Stroke::new(1.6_f32, ink());
        ui.painter()
            .line_segment([c - vec2(6., 0.), c + vec2(6., 0.)], stroke);
        ui.painter()
            .line_segment([c - vec2(0., 6.), c + vec2(0., 6.)], stroke);
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            if more {
                tr("Tuỳ chọn")
            } else {
                tr("Macro mới")
            },
        )
    });
    response
}

/// Visual neighbors move exactly one slot; the dragged action owns the open slot.
fn reorder_displacement(index: usize, from: usize, to: usize) -> i8 {
    if from < to && index > from && index <= to {
        -1
    } else if to < from && index >= to && index < from {
        1
    } else {
        0
    }
}

type ListInteraction = (Option<(usize, usize)>, Option<usize>);
fn animated_action_list(
    ui: &mut egui::Ui,
    width: f32,
    selected: u8,
    actions: &mut [MacroAction],
    recording: bool,
    selected_action: Option<usize>,
    mut draw: impl FnMut(&mut egui::Ui, f32, &mut MacroAction, bool) -> (Rect, egui::Response),
) -> ListInteraction {
    let mut move_action = None;
    let mut clicked = None;
    ui.spacing_mut().item_spacing.y = 8.;
    let top = ui.cursor().top();
    let step = 60.;
    let payload = egui::DragAndDrop::payload::<MacroDrag>(ui.ctx())
        .filter(|p| p.slot == selected && p.index < actions.len());
    let target = payload
        .as_ref()
        .and_then(|_| ui.ctx().pointer_interact_pos())
        .map(|p| {
            (((p.y - top - 26.) / step).round() as isize)
                .clamp(0, actions.len().saturating_sub(1) as isize) as usize
        });
    if let (Some(payload), Some(target)) = (&payload, target) {
        let placeholder = Rect::from_min_size(
            pos2(ui.cursor().left(), top + target as f32 * step),
            vec2(width, 52.),
        );
        ui.painter().rect(
            placeholder,
            5.,
            lilac().gamma_multiply(0.5),
            Stroke::new(1.0_f32, line()),
            StrokeKind::Inside,
        );
        let in_list = ui.ctx().pointer_interact_pos().is_some_and(|p| {
            p.x >= placeholder.left() && p.x <= placeholder.right() && ui.clip_rect().contains(p)
        });
        if in_list && ui.input(|i| i.pointer.any_released()) {
            move_action = Some((payload.index, target + usize::from(target > payload.index)));
            egui::DragAndDrop::clear_payload(ui.ctx());
        }
    }
    for (index, a) in actions.iter_mut().enumerate() {
        let id = ui.make_persistent_id(("macro_drag", selected, index));
        let dragging = ui.ctx().is_being_dragged(id);
        let active = selected_action == Some(index) || dragging;
        let (slot, _) = ui.allocate_exact_size(vec2(width, 52.), Sense::hover());
        let displacement = payload.as_ref().zip(target).map_or(0., |(p, to)| {
            reorder_displacement(index, p.index, to) as f32 * step
        });
        let y = ui.ctx().animate_value_with_time(
            id.with("slide"),
            displacement,
            if payload.is_some() { 0.16 } else { 0. },
        );
        let mut row_ui = ui.new_child(egui::UiBuilder::new().max_rect(slot.translate(vec2(0., y))));
        let source = if recording {
            row_ui.scope(|ui| draw(ui, width, a, active))
        } else {
            row_ui.dnd_drag_source(
                id,
                MacroDrag {
                    slot: selected,
                    index,
                },
                |ui| draw(ui, width, a, active),
            )
        };
        let (_, response) = source.inner;
        if dragging {
            // Read input before data_mut: both access the same Context lock.
            let press_offset = ui
                .input(|i| i.pointer.press_origin())
                .unwrap_or(slot.center())
                - slot.min;
            let grab = ui.ctx().data_mut(|data| {
                *data.get_temp_mut_or_insert_with(id.with("grab"), || press_offset)
            });
            let correction = slot.size() * 0.5 - grab;
            ui.ctx().transform_layer_shapes(
                egui::LayerId::new(egui::Order::Tooltip, id),
                egui::emath::TSTransform::from_translation(correction),
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else {
            ui.ctx()
                .data_mut(|data| data.remove::<Vec2>(id.with("grab")));
        }
        if response.clicked() && !recording {
            clicked = Some(index);
        }
    }
    (move_action, clicked)
}

#[cfg(test)]
mod drag_layout_tests {
    use super::*;
    #[test]
    fn pointer_drag_produces_a_drop_without_locking_context() {
        let ctx = egui::Context::default();
        let mut actions = vec![
            MacroAction {
                key: 4,
                status: 0x90,
                delay: 0
            };
            3
        ];
        let mut dropped = None;
        let button = |pressed, pos| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        };
        let frames = [
            vec![],
            vec![
                egui::Event::PointerMoved(pos2(20., 30.)),
                button(true, pos2(20., 30.)),
            ],
            vec![egui::Event::PointerMoved(pos2(20., 90.))],
            vec![egui::Event::PointerMoved(pos2(20., 150.))],
            vec![],
            vec![button(false, pos2(20., 150.))],
        ];
        for (frame, events) in frames.into_iter().enumerate() {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(600., 400.))),
                time: Some(frame as f64 * 0.1),
                events,
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (moved, _) = animated_action_list(
                        ui,
                        400.,
                        0,
                        &mut actions,
                        false,
                        None,
                        |ui, width, _, _| ui.allocate_exact_size(vec2(width, 52.), Sense::click()),
                    );
                    if moved.is_some() {
                        dropped = moved;
                    }
                });
            });
        }
        assert_eq!(dropped, Some((0, 3)));
    }
    #[test]
    fn only_neighbors_between_source_and_destination_shift() {
        assert_eq!(
            (0..5)
                .map(|i| reorder_displacement(i, 1, 3))
                .collect::<Vec<_>>(),
            [0, 0, -1, -1, 0]
        );
        assert_eq!(
            (0..5)
                .map(|i| reorder_displacement(i, 3, 1))
                .collect::<Vec<_>>(),
            [0, 1, 1, 0, 0]
        );
        assert!((0..5).all(|i| reorder_displacement(i, 2, 2) == 0));
    }
}
