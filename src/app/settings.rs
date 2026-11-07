use super::*;
impl NutApp {
    pub(super) fn device_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        if self.login_status_checked.elapsed() >= Duration::from_secs(3) {
            self.login_status =
                crate::autostart::status().unwrap_or(crate::autostart::Status::Unavailable);
            self.login_status_checked = Instant::now();
        }
        ui.ctx().request_repaint_after(Duration::from_secs(3));
        let body = page_body(ui, r, tr("Cài đặt"));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
        child.set_clip_rect(body);
        egui::ScrollArea::vertical()
            .id_salt("settings")
            .show(&mut child, |ui| {
                let content_width = ui.available_width();
                let mut card_rects = Vec::new();
                let backgrounds = [
                    ui.painter().add(egui::Shape::Noop),
                    ui.painter().add(egui::Shape::Noop),
                ];
                ui.columns(2, |cols| {
                    let left = egui::Frame::new()
                        .inner_margin(24)
                        .show(&mut cols[0], |ui| {
                            ui.set_width(((content_width - 10.) / 2. - 48.).max(0.));
                            ui.label(RichText::new(tr("Giao diện")).size(17.).strong());
                            ui.add_space(18.);
                            let old_language = self.library.language;
                            let old_theme = self.library.appearance;
                            setting_row(ui, tr("Ngôn ngữ"), |ui| {
                                combo_box("language")
                                    .width(ui.available_width() - 8.)
                                    .selected_text(self.library.language.name())
                                    .show_ui(ui, |ui| {
                                        for language in model::Language::ALL {
                                            dropdown_value(
                                                ui,
                                                &mut self.library.language,
                                                language,
                                                language.name(),
                                            );
                                        }
                                    });
                            });
                            setting_row(ui, tr("Giao diện"), |ui| {
                                let name = match self.library.appearance {
                                    model::Appearance::System => "Theo hệ thống",
                                    model::Appearance::Light => "Sáng",
                                    model::Appearance::Dark => "Tối",
                                };
                                combo_box("appearance")
                                    .width(ui.available_width() - 8.)
                                    .selected_text(tr(name))
                                    .show_ui(ui, |ui| {
                                        for (value, name) in [
                                            (model::Appearance::System, "Theo hệ thống"),
                                            (model::Appearance::Light, "Sáng"),
                                            (model::Appearance::Dark, "Tối"),
                                        ] {
                                            dropdown_value(
                                                ui,
                                                &mut self.library.appearance,
                                                value,
                                                tr(name),
                                            );
                                        }
                                    });
                            });
                            if old_language != self.library.language
                                || old_theme != self.library.appearance
                            {
                                self.changed();
                                ui.ctx().request_repaint();
                            }
                            ui.add_space(20.);
                            ui.separator();
                            ui.add_space(16.);
                            let old = self.library.colorway;
                            setting_row(ui, tr("Màu bàn phím"), |ui| {
                                combo_box("case_color")
                                    .width(ui.available_width() - 8.)
                                    .selected_text(self.library.colorway.name())
                                    .show_ui(ui, |ui| {
                                        let row_width = ui.available_width().max(240.);
                                        ui.set_width(row_width);
                                        for variant in Colorway::ALL {
                                            let (row, response) = ui.allocate_exact_size(
                                                vec2(row_width, 62.),
                                                Sense::click(),
                                            );
                                            let selected = variant == self.library.colorway;
                                            if selected || response.hovered() {
                                                ui.painter().rect_filled(row, 4., lilac());
                                            }
                                            let thumbnail = Rect::from_center_size(
                                                row.left_center() + vec2(67., 0.),
                                                vec2(126., 50.4),
                                            );
                                            preview::paint(
                                                ui.painter(),
                                                thumbnail,
                                                &self.areas,
                                                variant,
                                                None,
                                                self.keyboard.id(),
                                                &Default::default(),
                                            );
                                            text(
                                                ui.painter(),
                                                row.left_center() + vec2(140., 0.),
                                                variant.name(),
                                                14.,
                                                if selected { purple() } else { ink() },
                                                Align2::LEFT_CENTER,
                                            );
                                            response.widget_info(|| {
                                                egui::WidgetInfo::selected(
                                                    egui::WidgetType::SelectableLabel,
                                                    true,
                                                    selected,
                                                    variant.name(),
                                                )
                                            });
                                            if response.clicked() {
                                                self.library.colorway = variant;
                                                ui.close();
                                            }
                                        }
                                    });
                            });
                            if old != self.library.colorway {
                                self.changed();
                            }
                            let old = self.library.active;
                            setting_row(ui, tr("Hồ sơ"), |ui| {
                                combo_box("active_profile")
                                    .width(ui.available_width() - 8.)
                                    .selected_text(profile_display_name(&self.profile().name))
                                    .show_ui(ui, |ui| {
                                        for (i, profile) in self.library.profiles.iter().enumerate()
                                        {
                                            dropdown_value(
                                                ui,
                                                &mut self.library.active,
                                                i,
                                                profile_display_name(&profile.name),
                                            );
                                        }
                                        ui.separator();
                                        if ui.button(tr("Tên hồ sơ")).clicked() {
                                            self.tab = Tab::Profiles;
                                            ui.close();
                                        }
                                        if ui.button(tr("Nhập")).clicked() {
                                            self.import_profile();
                                            ui.close();
                                        }
                                        if ui.button(tr("Xuất")).clicked() {
                                            self.export_profile();
                                            ui.close();
                                        }
                                    });
                            });
                            if old != self.library.active {
                                self.profile_name = self.profile().name.clone();
                                self.changed();
                            }
                            ui.add_space(16.);
                            ui.label(RichText::new(tr("Hiện gán phím trên bàn phím")).size(14.));
                            ui.add_space(6.);
                            ui.horizontal(|ui| {
                                for (value, label) in [(false, "Ẩn"), (true, "Hiện")] {
                                    if assignment_choice(
                                        ui,
                                        label,
                                        value == self.show_labels,
                                        value,
                                    )
                                    .clicked()
                                    {
                                        self.show_labels = value;
                                    }
                                }
                            });
                            ui.add_space(12.);
                            let active = matches!(
                                self.login_status,
                                crate::autostart::Status::Enabled
                                    | crate::autostart::Status::RequiresApproval
                            );
                            let mut value = u8::from(active);
                            ui.add_enabled_ui(
                                self.login_status != crate::autostart::Status::Unavailable,
                                |ui| {
                                    device_switch(ui, tr("Chạy khi đăng nhập"), &mut value);
                                },
                            );
                            if (value != 0) != active {
                                match crate::autostart::set_enabled(value != 0) {
                                    Ok(status) => self.login_status = status,
                                    Err(error) => {
                                        eprintln!("{error:#}");
                                        self.login_status = crate::autostart::status()
                                            .unwrap_or(crate::autostart::Status::Unavailable);
                                        self.tell("Không thay đổi được cài đặt khởi động", true);
                                    }
                                }
                                self.login_status_checked = Instant::now();
                            }
                            if self.login_status == crate::autostart::Status::RequiresApproval {
                                ui.label(
                                    RichText::new(tr(
                                        "Cho phép ứng dụng trong cài đặt đăng nhập của hệ thống.",
                                    ))
                                    .size(13.)
                                    .color(muted()),
                                );
                            }
                        });
                    card_rects.push(left.response.rect);
                    let right = egui::Frame::new()
                        .inner_margin(24)
                        .show(&mut cols[1], |ui| {
                            ui.set_width(((content_width - 10.) / 2. - 48.).max(0.));
                            ui.label(RichText::new("WEIKAV NUT75").size(17.).strong());
                            ui.add_space(12.);
                            let width = ui.available_width();
                            // Keep the preview proportional, with enough room for all device details.
                            let height = (width * 0.4).min((body.height() - 300.).max(96.));
                            let (slot, _) =
                                ui.allocate_exact_size(vec2(width, height), Sense::hover());
                            let rect =
                                Rect::from_center_size(slot.center(), vec2(height / 0.4, height));
                            preview::paint(
                                ui.painter(),
                                rect,
                                &self.areas,
                                self.library.colorway,
                                None,
                                self.keyboard.id(),
                                &Default::default(),
                            );
                            ui.scope(|ui| {
                                ui.spacing_mut().item_spacing.y = 4.;
                                ui.style_mut()
                                    .text_styles
                                    .insert(egui::TextStyle::Body, FontId::proportional(13.));
                                ui.add_space(8.);
                                if let Some(snapshot) = &self.snapshot {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(tr("Đã kết nối"))
                                                .size(13.)
                                                .color(green()),
                                        );
                                        ui.label(
                                            if snapshot.entry.product_id
                                                == crate::device::PRODUCT_USB
                                            {
                                                "USB-C"
                                            } else {
                                                "2.4 GHz"
                                            },
                                        );
                                    });
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            tr("Firmware"),
                                            snapshot.info.version
                                        ))
                                        .size(12.)
                                        .color(muted()),
                                    );
                                    ui.add_space(8.);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(!self.busy, button("Đọc lại", false))
                                            .clicked()
                                        {
                                            self.send(Command::Refresh);
                                        }
                                        if ui
                                            .add_enabled(!self.busy, button("Ngắt kết nối", false))
                                            .clicked()
                                        {
                                            self.send(Command::Disconnect);
                                        }
                                    });
                                } else if ui
                                    .add_enabled(!self.busy, button("Kết nối thiết bị", true))
                                    .clicked()
                                {
                                    self.connect_open = true;
                                    self.send(Command::Scan);
                                }
                                ui.add_space(8.);
                                ui.separator();
                                ui.add_space(4.);
                                ui.label(
                                    RichText::new(tr("Chi tiết thiết bị"))
                                        .size(12.)
                                        .color(muted()),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "Weikav Studio {}",
                                        env!("CARGO_PKG_VERSION")
                                    ))
                                    .size(12.)
                                    .color(muted()),
                                );
                                if let Some(s) = &self.snapshot {
                                    ui.label(
                                        RichText::new(format!(
                                            "VID / PID: {:04X} / {:04X}",
                                            s.entry.vendor_id, s.entry.product_id
                                        ))
                                        .size(12.)
                                        .color(muted()),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "HID: {} byte - v{}",
                                            s.report_size, s.info.frame_version
                                        ))
                                        .size(12.)
                                        .color(muted()),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} / {} - {} {} - {} {}",
                                            s.info.manufacturer,
                                            s.info.product,
                                            tr("Hồ sơ thiết bị"),
                                            s.info.profile + 1,
                                            tr("Chế độ"),
                                            s.info.work_mode
                                        ))
                                        .size(12.)
                                        .color(muted()),
                                    );
                                }
                            });
                        });
                    card_rects.push(right.response.rect);
                });
                let height = card_rects
                    .iter()
                    .map(|r| r.height())
                    .fold(0.0_f32, f32::max);
                for (idx, r) in backgrounds.into_iter().zip(card_rects) {
                    let rect = Rect::from_min_size(r.min, vec2(r.width(), height));
                    ui.painter().set(
                        idx,
                        egui::Shape::Vec(vec![
                            egui::Shape::rect_filled(rect, 6., surface()),
                            egui::Shape::rect_stroke(
                                rect,
                                6.,
                                Stroke::new(1.0_f32, line()),
                                StrokeKind::Inside,
                            ),
                        ]),
                    );
                }
            });
    }
}
fn assignment_choice(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    shown: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(114., 48.), Sense::click());
    ui.painter().rect(
        rect,
        6.,
        if selected { lilac() } else { surface() },
        Stroke::new(
            if selected { 1.5_f32 } else { 1. },
            if selected { purple() } else { line() },
        ),
        StrokeKind::Inside,
    );
    let center = rect.left_center() + vec2(22., 0.);
    icon(
        ui.painter(),
        center,
        0,
        if selected { purple() } else { muted() },
        0.9,
    );
    if !shown {
        ui.painter().line_segment(
            [center + vec2(-9., 9.), center + vec2(9., -9.)],
            Stroke::new(1.6_f32, if selected { purple() } else { muted() }),
        );
    }
    text(
        ui.painter(),
        rect.left_center() + vec2(42., 0.),
        label,
        14.,
        if selected { purple() } else { muted() },
        Align2::LEFT_CENTER,
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, tr(label))
    });
    response
}
fn setting_row(ui: &mut egui::Ui, label: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [128., 36.],
            egui::Label::new(RichText::new(label).size(14.)),
        );
        ui.allocate_ui_with_layout(
            vec2((ui.available_width()).max(150.), 36.),
            egui::Layout::left_to_right(egui::Align::Center),
            content,
        );
    });
    ui.add_space(10.);
}

impl NutApp {
    pub(super) fn keyboard_settings_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        let body = page_body(ui, r, tr("Cài đặt bàn phím"));
        let wide = body.width() >= 860.;
        let gap = 24.;
        let controls_width = if wide {
            (body.width() * 0.48).clamp(430., 560.)
        } else {
            body.width()
        };
        let controls =
            Rect::from_min_max(pos2(body.right() - controls_width, body.top()), body.max);
        if wide {
            let preview_rect =
                Rect::from_min_max(body.min, pos2(controls.left() - gap, body.bottom()));
            let center = pos2(preview_rect.center().x, preview_rect.top() + 220.);
            let width = preview_rect.width().min(680.);
            let keyboard = Rect::from_center_size(center, vec2(width, width * 0.4));
            preview::paint(
                ui.painter(),
                keyboard,
                &self.areas,
                self.library.colorway,
                None,
                self.keyboard.id(),
                &Default::default(),
            );
            text(
                ui.painter(),
                pos2(center.x, keyboard.bottom() + 30.),
                "WEIKAV NUT75",
                18.,
                ink(),
                Align2::CENTER_CENTER,
            );
            if let Some(snapshot) = &self.snapshot {
                let connection = if snapshot.entry.product_id == crate::device::PRODUCT_USB {
                    "USB-C"
                } else {
                    "2.4 GHz"
                };
                text(
                    ui.painter(),
                    pos2(center.x, keyboard.bottom() + 56.),
                    &format!("{} - {connection}", tr("Đã kết nối")),
                    13.,
                    muted(),
                    Align2::CENTER_CENTER,
                );
            }
        }
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(controls));
        child.set_clip_rect(controls);
        egui::ScrollArea::vertical()
            .id_salt("device_settings_page")
            .show(&mut child, |ui| {
                egui::Frame::new()
                    .fill(surface())
                    .stroke(Stroke::new(1.0_f32, line()))
                    .corner_radius(8.)
                    .inner_margin(24)
                    .show(ui, |ui| {
                        ui.set_width(controls_width - 48.);
                        ui.spacing_mut().item_spacing.y = 8.;
                        if self.snapshot.is_some() {
                            self.keyboard_settings(ui);
                        } else {
                            ui.label(
                                RichText::new(tr("Cấu hình qua USB-C hoặc receiver 2.4 GHz."))
                                    .size(14.)
                                    .color(muted()),
                            );
                            ui.add_space(12.);
                            if ui
                                .add_enabled(!self.busy, button("Kết nối thiết bị", true))
                                .clicked()
                            {
                                self.connect_open = true;
                                self.send(Command::Scan);
                            }
                        }
                    });
            });
    }
}
pub(super) fn device_switch(ui: &mut egui::Ui, label: &str, value: &mut u8) {
    let (r, response) = ui.allocate_exact_size(vec2(ui.available_width(), 42.), Sense::click());
    let on = *value != 0;
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, on, label));
    text(
        ui.painter(),
        r.left_center(),
        label,
        14.,
        ink(),
        Align2::LEFT_CENTER,
    );
    let track = Rect::from_center_size(r.right_center() - vec2(20., 0.), vec2(36., 20.));
    ui.painter()
        .rect_filled(track, 10., if on { brand() } else { shade(224) });
    ui.painter().circle_filled(
        pos2(
            if on {
                track.right() - 10.
            } else {
                track.left() + 10.
            },
            track.center().y,
        ),
        8.,
        Color32::WHITE,
    );
    if response.clicked() {
        *value = u8::from(!on);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand);
}

/// Labels share the left edge; dropdowns, sliders and switches share the right edge.
pub(super) fn device_control_row(
    ui: &mut egui::Ui,
    label: &str,
    control: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.set_min_height(42.);
        ui.label(RichText::new(label).size(14.));
        let width = 184.;
        let left = (ui.max_rect().right() - width).max(ui.cursor().left());
        let rect = Rect::from_min_size(pos2(left, ui.cursor().top()), vec2(width, 42.));
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            control,
        );
    });
}
