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
        self.keyboard_stage(
            ui,
            Rect::from_min_max(r.min, pos2(r.right(), r.bottom() - 170.)),
        );
        let rect = Rect::from_min_max(
            pos2(r.left() + 28., r.bottom() - 170.),
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
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for (function, label) in [(true, "Fn"), (false, "Mặc định")] {
                    if ui.add(button(label, self.function == function)).clicked() {
                        self.function = function;
                        self.selected = None;
                    }
                }
            });
        });
        egui::ScrollArea::vertical()
            .id_salt("advanced_bindings")
            .max_height(112.)
            .show(&mut child, |ui| {
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
                        let label = format!(
                            "{} - {}   {}",
                            if function { "Fn" } else { tr("Mặc định") },
                            self.key_name(id),
                            tr(&model::binding_label(b, ""))
                        );
                        if ui
                            .add_sized(
                                [ui.available_width() - 62., 32.],
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
                        if ui.add_enabled(!self.busy, button("Xoá", false)).clicked() {
                            self.function = function;
                            self.selected = Some(id);
                            self.assign(DEFAULT);
                        }
                    });
                }
            });
    }
    pub(super) fn advanced_panel(&mut self, ui: &mut egui::Ui) {
        if self.selected.is_some() {
            self.key_inspector(ui);
            return;
        }
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
