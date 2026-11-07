use super::*;
const TYPES: [(&str, &str); 4] = [
    (
        "MT - Chạm / giữ",
        "Chạm một chức năng, giữ một chức năng khác.",
    ),
    ("TGL - Bật / tắt", "Nhấn để bật hoặc tắt giữ phím."),
    (
        "SOCD - Hai phím đối hướng",
        "Chọn cách ưu tiên giữa hai phím đối hướng.",
    ),
    ("CB - Tổ hợp phím", "Gán phím cùng các phím bổ trợ."),
];
impl NutApp {
    pub(super) fn load_advanced_binding(&mut self, b: Binding) {
        match b {
            [9, a, b, t] => {
                self.advanced_kind = 0;
                self.advanced_keys = [a, b, 6];
                self.advanced_delay = u16::from(t) * 10;
            }
            [10, a, ..] => {
                self.advanced_kind = 1;
                self.advanced_keys[0] = a;
            }
            [11, mode, _, second] => {
                self.advanced_kind = 2;
                self.advanced_mode = mode;
                self.advanced_keys[2] = second;
            }
            [7, a, b, c] => {
                self.advanced_kind = 3;
                self.advanced_keys = [a, b, c];
            }
            _ => {}
        }
    }
    pub(super) fn advanced_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        // Reserve space for the list, then start it just below the actual keyboard
        // rather than at a fixed offset from the bottom of the window.
        let list_reserve = (r.height() * 0.35).clamp(200., 300.);
        let keyboard_rect = Rect::from_min_max(r.min, pos2(r.right(), r.bottom() - list_reserve));
        self.keyboard_stage(ui, keyboard_rect);
        let viewport = Rect::from_min_max(
            keyboard_rect.min + vec2(28., 56.),
            keyboard_rect.max - vec2(28., 64.),
        );
        let width = viewport
            .width()
            .min(viewport.height() / 0.4)
            .clamp(1., 1100.);
        let keyboard_bottom = viewport.center().y - width * 0.2 + width * 0.386;
        let rect = Rect::from_min_max(
            pos2(
                r.left() + 28.,
                (keyboard_bottom + 20.).min(r.bottom() - 100.),
            ),
            r.max - vec2(28., 0.),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect);
        let mut bindings = Vec::new();
        for function in [false, true] {
            for area in &self.areas {
                let b = self
                    .profile()
                    .patches(function)
                    .get(&area.id)
                    .copied()
                    .or_else(|| self.snapshot.as_ref().map(|s| s.binding(function, area.id)))
                    .unwrap_or(DEFAULT);
                if matches!(b[0], 7 | 9 | 10 | 11) {
                    bindings.push((function, area.id, b));
                }
            }
        }
        child.horizontal(|ui| {
            ui.label(RichText::new(tr("Phím đã gán")).size(14.).strong());
            ui.label(
                RichText::new(bindings.len().to_string())
                    .size(13.)
                    .color(muted()),
            );
            ui.add_space(12.);
            for (function, label) in [(false, "Mặc định"), (true, "Fn")] {
                if layer_choice(ui, label, self.function == function, function).clicked() {
                    self.function = function;
                    self.selected = None;
                }
            }
        });
        let list_height = child.available_height().max(0.);
        // Keep the content width stable while egui animates the scrollbar in and out.
        let scrollbar_space = child.spacing().scroll.allocated_width();
        let list_width = (rect.width() - scrollbar_space - 4.).max(0.);
        egui::ScrollArea::vertical()
            .id_salt("advanced_bindings")
            .auto_shrink([false, false])
            .max_height(list_height)
            .show(&mut child, |ui| {
                ui.set_width(list_width);
                ui.spacing_mut().item_spacing.y = 4.;
                if bindings.is_empty() {
                    ui.label(
                        RichText::new(tr("Chọn một phím trên bàn phím để gán chức năng."))
                            .size(13.)
                            .color(muted()),
                    );
                }
                for (function, id, b) in bindings {
                    ui.horizontal(|ui| {
                        ui.set_width(list_width);
                        let label = format!(
                            "{} - {}   {}",
                            if function { "Fn" } else { tr("Mặc định") },
                            self.key_name(id),
                            tr(&model::binding_label(b, ""))
                        );
                        if ui
                            .add_sized(
                                [
                                    (ui.available_width() - 36. - ui.spacing().item_spacing.x)
                                        .max(0.),
                                    36.,
                                ],
                                button(
                                    &label,
                                    self.selected == Some(id) && self.function == function,
                                ),
                            )
                            .clicked()
                        {
                            self.function = function;
                            self.selected = Some(id);
                            self.load_advanced_binding(b);
                        }
                        let more = ui
                            .add_enabled_ui(!self.busy, |ui| {
                                super::macro_editor::macro_header_tool(ui, true)
                                    .on_hover_text(tr("Tuỳ chọn"))
                            })
                            .inner;
                        egui::Popup::from_toggle_button_response(&more)
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .width(160.)
                            .frame(
                                egui::Frame::popup(ui.style())
                                    .fill(surface())
                                    .stroke(Stroke::new(1.0_f32, line()))
                                    .corner_radius(6.)
                                    .inner_margin(6),
                            )
                            .show(|ui| {
                                ui.set_width(148.);
                                if super::macro_editor::menu_row(ui, tr("Xoá"), !self.busy, false)
                                    .clicked()
                                {
                                    self.function = function;
                                    self.selected = Some(id);
                                    self.assign(DEFAULT);
                                    ui.close();
                                }
                            });
                    });
                }
            });
    }
    pub(super) fn advanced_panel(&mut self, ui: &mut egui::Ui) {
        if self.selected.is_some() {
            self.key_inspector(ui);
            return;
        }
        let function = self.function;
        let ids: Vec<u8> = self
            .areas
            .iter()
            .filter(|area| !self.locked(area.id))
            .filter(|area| matches!(self.binding(area.id)[0], 7 | 9 | 10 | 11))
            .map(|area| area.id)
            .collect();
        let reset_label = format!(
            "{} - {}",
            tr("Đặt lại phím nâng cao"),
            tr(if function { "Fn" } else { "Mặc định" })
        );
        let reset_width = ui.available_width();
        if ui
            .add_enabled_ui(!self.busy && !ids.is_empty(), |ui| {
                ui.add_sized([reset_width, 36.], button(&reset_label, false))
            })
            .inner
            .on_hover_text(tr("Chỉ áp dụng cho lớp đang chọn"))
            .clicked()
        {
            for id in ids {
                self.profile_mut().patches_mut(function).insert(id, DEFAULT);
            }
            self.changed();
        }
        ui.add_space(10.);
        for (kind, (title, description)) in TYPES.iter().enumerate() {
            let (r, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), 86.), Sense::click());
            ui.painter().rect(
                r,
                6.,
                if self.advanced_kind == kind {
                    lilac()
                } else {
                    surface()
                },
                Stroke::new(1.0_f32, line()),
                StrokeKind::Inside,
            );
            ui.painter().image(
                self.advanced_icons[kind].id(),
                Rect::from_center_size(r.left_center() + vec2(28., 0.), vec2(30., 30.)),
                Rect::from_min_max(Pos2::ZERO, pos2(1., 1.)),
                if self.advanced_kind == kind {
                    purple()
                } else {
                    ink()
                },
            );
            text(
                ui.painter(),
                r.min + vec2(54., 22.),
                tr(title),
                14.,
                if self.advanced_kind == kind {
                    purple()
                } else {
                    ink()
                },
                Align2::LEFT_CENTER,
            );
            let galley = ui.painter().layout(
                tr(description).into(),
                FontId::proportional(12.),
                muted(),
                r.width() - 66.,
            );
            ui.painter().galley(r.min + vec2(54., 42.), galley, muted());
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    true,
                    self.advanced_kind == kind,
                    tr(title),
                )
            });
            if response.clicked() {
                self.advanced_kind = kind;
            }
        }
        ui.add_space(8.);
        ui.label(
            RichText::new(tr("Chọn một phím trên bàn phím để gán chức năng."))
                .size(13.)
                .color(muted()),
        );
    }
}
