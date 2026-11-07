use super::*;

fn color_mode_card(
    ui: &mut egui::Ui,
    selected: bool,
    enabled: bool,
    rainbow: bool,
    color: [u8; 3],
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(112., 48.),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let painter = ui.painter();
    painter.rect(
        rect,
        6.,
        if selected {
            purple().gamma_multiply(0.12)
        } else {
            surface()
        },
        Stroke::new(
            if selected { 1.5_f32 } else { 1. },
            if selected { purple() } else { line() },
        ),
        StrokeKind::Inside,
    );
    let colors = [
        Color32::from_rgb(255, 104, 118),
        Color32::from_rgb(255, 179, 85),
        Color32::from_rgb(239, 218, 93),
        Color32::from_rgb(104, 210, 156),
        Color32::from_rgb(90, 196, 225),
        Color32::from_rgb(166, 117, 245),
    ];
    let origin = rect.min + vec2(10., 13.);
    for row in 0..3 {
        for col in 0..6 {
            let fill = if rainbow {
                colors[(col + row) % colors.len()]
            } else {
                Color32::from_rgb(color[0], color[1], color[2])
            };
            let key = Rect::from_min_size(
                origin + vec2(col as f32 * 5.4, row as f32 * 7.5),
                vec2(4.3, 5.7),
            );
            painter.rect(
                key,
                1.,
                fill,
                Stroke::new(0.5_f32, muted().gamma_multiply(0.5)),
                StrokeKind::Inside,
            );
        }
    }
    text(
        painter,
        pos2(rect.left() + 49., rect.center().y),
        if rainbow { "RGB" } else { "Đơn" },
        14.,
        if selected { purple() } else { muted() },
        Align2::LEFT_CENTER,
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            enabled,
            selected,
            tr(if rainbow { "RGB" } else { "Đơn" }),
        )
    });
    response
}

impl NutApp {
    pub(super) fn ai_lighting_supported(&self) -> bool {
        self.snapshot.as_ref().is_some_and(|s| {
            s.entry.product_id == crate::device::PRODUCT_RECEIVER
                && s.report_size == 64
                && s.info.lighting_version == 1
        })
    }

    pub(super) fn animation_supported(&self) -> bool {
        self.snapshot.as_ref().is_some_and(|s| {
            s.entry.product_id == crate::device::PRODUCT_RECEIVER
                && s.info.gif_max_frames > 0
                && self.animations[self.animation_selected].data.len() / 81
                    <= usize::from(s.info.gif_max_frames)
        })
    }

    pub(super) fn stop_ai_lighting(&mut self) {
        if self.ai_playing {
            self.ai_playing = false;
            if let Some(original) = self.ai_original_light.take() {
                self.send(Command::StopColors(original));
            }
        }
    }

    pub(super) fn ai_lighting_inspector(&mut self, ui: &mut egui::Ui) {
        for (id, name) in ["Cầu vồng", "Pháo hoa", "Trái tim"].into_iter().enumerate() {
            if ui
                .add_sized(
                    [ui.available_width(), 46.],
                    button(name, self.ai_effect == id),
                )
                .clicked()
            {
                self.ai_effect = id;
            }
        }
        ui.add_space(12.);
        ui.label(
            RichText::new(tr("Chạy khi ứng dụng đang mở"))
                .size(13.)
                .color(muted()),
        );
        if !self.ai_lighting_supported() && self.snapshot.is_some() {
            ui.label(
                RichText::new(tr("Cần kết nối 2.4 GHz hỗ trợ"))
                    .size(13.)
                    .color(muted()),
            );
        }
    }

    pub(super) fn animation_inspector(&mut self, ui: &mut egui::Ui) {
        let mut import = false;
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.), Sense::hover());
        {
            let rect = Rect::from_min_size(pos2(row.right() - 36., row.top()), vec2(36., 36.));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
            let ui = &mut child;
            let more =
                super::macro_editor::macro_header_tool(ui, true).on_hover_text(tr("Tuỳ chọn"));
            egui::Popup::from_toggle_button_response(&more)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .width(200.)
                .frame(
                    egui::Frame::popup(ui.style())
                        .fill(surface())
                        .stroke(Stroke::new(1.0_f32, line()))
                        .corner_radius(6.)
                        .inner_margin(6),
                )
                .show(|ui| {
                    ui.set_width(188.);
                    if super::macro_editor::menu_row(ui, tr("Nhập JSON"), true, false).clicked() {
                        import = true;
                        ui.close();
                    }
                });
        }
        ui.add_space(8.);
        let names = [
            "Trái tim",
            "Băng hà",
            "Mùa xuân",
            "Ngọc bích",
            "Trắng",
            "Mận",
            "Cam",
        ];
        for (index, animation) in self.animations.iter().enumerate() {
            let name = names.get(index).copied().unwrap_or(&animation.name);
            if ui
                .add_sized(
                    [ui.available_width(), 38.],
                    button(name, self.animation_selected == index),
                )
                .clicked()
            {
                self.animation_selected = index;
                self.preview_time = 0.;
            }
        }
        if import
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Animation", &["json"])
                .pick_file()
        {
            let imported = (|| -> anyhow::Result<crate::animation::Animation> {
                anyhow::ensure!(
                    std::fs::metadata(&path)?.len() <= 1_000_000,
                    "Animation file too large"
                );
                Ok(serde_json::from_slice(&std::fs::read(&path)?)?)
            })()
            .and_then(|animation| {
                animation.frames()?;
                Ok(animation)
            });
            match imported {
                Ok(animation) => {
                    self.animations.push(animation);
                    self.animation_selected = self.animations.len() - 1;
                }
                Err(error) => {
                    eprintln!("{error:#}");
                    self.tell("Không nhập được hồ sơ", true);
                }
            }
        }
    }

    pub(super) fn solid_color_supported(&self, mode: u8) -> bool {
        !matches!(mode, 0 | 6 | 8 | 23)
            && !(mode == 10
                && self
                    .snapshot
                    .as_ref()
                    .is_none_or(|s| s.info.version == "1.40"))
    }
    pub(super) fn lighting_controls_stage(&mut self, ui: &mut egui::Ui, rect: Rect) -> f32 {
        let mut light = self.current_lighting();
        let before = light.clone();
        let mut undo = false;
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect);
        let frame = egui::Frame::new()
            .fill(surface())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(6.)
            .inner_margin(14)
            .show(&mut child, |ui| {
                ui.set_width(rect.width() - 28.);
                ui.spacing_mut().item_spacing = vec2(10., 8.);
                ui.add_enabled_ui(!self.busy, |ui| {
                    ui.horizontal(|ui| {
                        let solid_color = self.solid_color_supported(light.mode);
                        if light.mode == 128 {
                            ui.label(RichText::new(tr("Màu")).size(14.));
                            self.edit_color(ui, "main", &mut light.color);
                            if ui.add(button("Tô toàn bộ", false)).clicked() {
                                for id in self.areas.iter().map(|a| a.id).collect::<Vec<_>>() {
                                    self.profile_mut().custom_colors.insert(id, light.color);
                                }
                                self.changed();
                            }
                        } else if light.mode != 0 {
                            for (rainbow, name) in [(true, "RGB"), (false, "Đơn")] {
                                let selected = if solid_color {
                                    light.rainbow == rainbow
                                } else {
                                    rainbow
                                };
                                let response = color_mode_card(
                                    ui,
                                    selected,
                                    solid_color,
                                    rainbow,
                                    light.color,
                                );
                                let response = if solid_color {
                                    response.on_hover_text(tr(name))
                                } else {
                                    response.on_hover_text(tr(if light.mode == 10 {
                                        "Cross weave trên firmware này chỉ chạy RGB."
                                    } else {
                                        "Hiệu ứng này chạy màu RGB tự động."
                                    }))
                                };
                                if response.clicked() {
                                    light.rainbow = rainbow;
                                }
                            }
                            if solid_color && !light.rainbow {
                                ui.label(RichText::new(tr("Màu")).size(14.));
                                self.edit_color(ui, "main", &mut light.color);
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            undo = ui
                                .add_enabled(
                                    self.profile().lighting.is_some()
                                        || !self.profile().custom_colors.is_empty(),
                                    egui::Button::new(
                                        RichText::new(tr("Hoàn tác")).size(14.).color(muted()),
                                    )
                                    .frame(false),
                                )
                                .clicked();
                        });
                    });
                    ui.columns(2, |cols| {
                        cols[0].label(RichText::new(tr("Độ sáng")).size(14.));
                        cols[0].spacing_mut().slider_width =
                            (cols[0].available_width() - 54.).max(70.);
                        cols[0].add_enabled_ui(light.mode != 0, |ui| {
                            options_slider(
                                ui,
                                egui::Slider::new(&mut light.brightness, 1..=6).integer(),
                            );
                        });
                        cols[1].label(RichText::new(tr("Tốc độ")).size(14.));
                        cols[1].spacing_mut().slider_width =
                            (cols[1].available_width() - 54.).max(70.);
                        cols[1].add_enabled_ui(!matches!(light.mode, 0 | 1 | 128), |ui| {
                            options_slider(
                                ui,
                                egui::Slider::new(&mut light.speed, 1..=6).integer(),
                            );
                        });
                    });
                    if model::light_has_direction(light.mode) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(tr("Hướng")).size(14.));
                            let choices: &[(u8, &str)] = if light.mode == 10 {
                                &[(3, "↑"), (2, "↓")]
                            } else {
                                &[(0, "←"), (1, "→")]
                            };
                            for &(value, name) in choices {
                                ui.selectable_value(&mut light.direction, value, name);
                            }
                        });
                    } else if light.mode == 128 {
                        ui.label(
                            RichText::new(tr("Chọn màu rồi bấm phím để tô."))
                                .size(14.)
                                .color(muted()),
                        );
                    }
                });
            });
        light.direction = light.normalized_direction();
        if before != light {
            self.profile_mut().lighting = Some(light);
            self.changed();
        }
        if undo {
            self.profile_mut().lighting = None;
            self.profile_mut().custom_colors.clear();
            self.changed();
        }
        frame.response.rect.bottom()
    }

    pub(super) fn lighting_shortcut_guide(&self, ui: &mut egui::Ui, rect: Rect) {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect);
        egui::Frame::new()
            .fill(surface())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(6.)
            .inner_margin(12)
            .show(&mut child, |ui| {
                ui.set_width(rect.width() - 24.);
                ui.spacing_mut().item_spacing.y = 6.;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr("Phím tắt theo hãng")).size(13.).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to(
                            RichText::new(tr("Hướng dẫn WEIKAV")).size(12.),
                            MANUAL_URL,
                        );
                    });
                });
                let rows = shortcut_rows(self.light_zone);
                egui::Grid::new(("lighting_shortcuts", self.light_zone))
                    .num_columns(2)
                    .spacing([12., 8.])
                    .show(ui, |ui| {
                        let width = ((rect.width() - 36.) / 2.).max(1.);
                        for (index, row) in rows.iter().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(width);
                                    ui.label(RichText::new(tr(row.label)).size(13.).color(muted()));
                                    let (keys, response) =
                                        ui.allocate_exact_size(vec2(width, 44.), Sense::hover());
                                    let mut x = keys.left();
                                    for (position, usage) in row.keys.iter().enumerate() {
                                        if position > 0 {
                                            text(
                                                ui.painter(),
                                                pos2(x + 8., keys.center().y),
                                                if position == 1 { "+" } else { "/" },
                                                13.,
                                                muted(),
                                                Align2::CENTER_CENTER,
                                            );
                                            x += 20.;
                                        }
                                        let w = self
                                            .areas
                                            .iter()
                                            .find(|a| a.key.key_code == *usage)
                                            .map(|a| {
                                                a.rect[2] * (40. / a.rect[3]).min(90. / a.rect[2])
                                            })
                                            .unwrap_or(32.);
                                        self.paint_keyboard_keycap_at_height(
                                            ui.painter(),
                                            pos2(x + w / 2., keys.center().y),
                                            *usage,
                                            false,
                                            40.,
                                        );
                                        x += w;
                                    }
                                    response.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::WidgetType::Label,
                                            true,
                                            format!("{}: {}", tr(row.label), row.accessible_keys),
                                        )
                                    });
                                    if let Some(note) = row.note {
                                        response.on_hover_text(tr(note));
                                    }
                                });
                            });
                            if index % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
            });
    }

    pub(super) fn light_zone_picker(&mut self, ui: &mut egui::Ui) -> bool {
        let (rail, _) = ui.allocate_exact_size(vec2(ui.available_width(), 38.), Sense::hover());
        ui.painter().rect(
            rail,
            19.,
            shade(240),
            Stroke::new(0.7_f32, line()),
            StrokeKind::Inside,
        );
        let segment = (rail.width() - 8.) / 3.;
        let animated = ui.ctx().animate_value_with_time(
            egui::Id::new("lighting_zone_lens"),
            self.light_zone as f32,
            0.16,
        );
        let lens = Rect::from_min_size(
            rail.min + vec2(4. + animated * segment, 4.),
            vec2(segment, 30.),
        );
        ui.painter().rect_filled(
            lens.translate(vec2(0., 1.)),
            15.,
            Color32::from_black_alpha(14),
        );
        ui.painter().rect(
            lens,
            15.,
            surface(),
            Stroke::new(0.8_f32, line()),
            StrokeKind::Inside,
        );
        for (id, name) in [(0, "Phím"), (1, "Caps Lock"), (2, "Hai bên")] {
            let r = Rect::from_min_size(
                rail.min + vec2(4. + id as f32 * segment, 0.),
                vec2(segment, 38.),
            );
            let response = ui.interact(r, egui::Id::new(("lighting_zone", id)), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    true,
                    self.light_zone == id,
                    tr(name),
                )
            });
            text(
                ui.painter(),
                r.center(),
                tr(name),
                14.,
                if self.light_zone == id {
                    ink()
                } else {
                    muted()
                },
                Align2::CENTER_CENTER,
            );
            if response.clicked() {
                self.light_zone = id;
                if id == 1 {
                    self.caps_color = self
                        .custom_colors()
                        .get(&48)
                        .copied()
                        .filter(|_| self.current_lighting().mode == 128)
                        .unwrap_or(self.current_lighting().color);
                }
            }
            response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
        ui.add_space(18.);
        match self.light_zone {
            1 => {
                self.caps_light_inspector(ui);
                true
            }
            2 => {
                self.side_light_inspector(ui);
                true
            }
            _ => false,
        }
    }

    fn caps_light_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(tr("Đèn phím Caps Lock")).size(16.).strong());
        ui.add_space(10.);
        ui.label(
            RichText::new(tr(
                "Màu riêng dùng chế độ Tự chọn từng phím cho toàn bàn phím.",
            ))
            .size(14.)
            .color(muted()),
        );
        ui.add_space(14.);
        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Màu"));
                let mut color = self.caps_color;
                let changed = self.edit_color(ui, "caps", &mut color);
                self.caps_color = color;
                if changed && self.current_lighting().mode == 128 {
                    let color = self.caps_color;
                    self.profile_mut().custom_colors.insert(48, color);
                    self.changed();
                }
            });
            ui.add_space(12.);
            if self.current_lighting().mode != 128
                && ui
                    .add_sized([ui.available_width(), 36.], button("Dùng màu riêng", true))
                    .clicked()
            {
                let mut light = self.current_lighting();
                // Preserve the visible solid color when entering custom mode, rather than
                // making every other key dark using the firmware's empty custom buffer.
                for id in self.areas.iter().map(|a| a.id).collect::<Vec<_>>() {
                    self.profile_mut()
                        .custom_colors
                        .entry(id)
                        .or_insert(light.color);
                }
                let color = self.caps_color;
                self.profile_mut().custom_colors.insert(48, color);
                light.mode = 128;
                self.profile_mut().lighting = Some(light);
                self.changed();
            }
        });
        ui.add_space(26.);
        ui.separator();
        ui.add_space(14.);
        ui.label(RichText::new(tr("Đèn báo Caps Lock")).size(16.).strong());
        ui.label(
            RichText::new(tr("Chưa hỗ trợ đổi màu đèn báo trong app."))
                .size(14.)
                .color(muted()),
        );
    }

    fn side_light_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(tr("Đèn hai bên")).size(16.).strong());
        ui.add_space(12.);
        let Some(mut side) = self
            .profile()
            .side_lighting
            .clone()
            .or_else(|| self.snapshot.as_ref().and_then(|s| s.side_lighting()))
        else {
            ui.label(
                RichText::new(tr("Chưa hỗ trợ chỉnh đèn hai bên trên firmware này."))
                    .size(14.)
                    .color(muted()),
            );
            ui.add_space(14.);
            if self.snapshot.is_some() {
                if ui
                    .add_enabled(!self.busy, button("Đọc lại", false))
                    .clicked()
                {
                    self.send(Command::Refresh);
                }
            } else if ui
                .add_enabled(!self.busy, button("Kết nối thiết bị", false))
                .clicked()
            {
                self.connect_open = true;
                self.send(Command::Scan);
            }
            return;
        };
        let supported = self
            .snapshot
            .as_ref()
            .and_then(|s| s.side_lighting())
            .is_some();
        let before = side.clone();
        ui.add_enabled_ui(!self.busy && supported, |ui| {
            let width = (ui.available_width() - 8.) / 2.;
            egui::Grid::new("side_effects")
                .num_columns(2)
                .spacing([8., 8.])
                .show(ui, |ui| {
                    for (i, (mode, name)) in model::SIDE_MODES.iter().enumerate() {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, 50.), Sense::click());
                        let active = side.mode == *mode;
                        ui.painter().rect(
                            rect,
                            6.,
                            if active { lilac() } else { surface() },
                            Stroke::new(1.0_f32, if active { purple() } else { line() }),
                            StrokeKind::Inside,
                        );
                        let mut preview = side.clone();
                        preview.mode = *mode;
                        effect_icon(
                            ui.painter(),
                            Rect::from_center_size(
                                pos2(rect.left() + 22., rect.center().y),
                                vec2(26., 26.),
                            ),
                            preview.preview().mode,
                            purple(),
                        );
                        text(
                            ui.painter(),
                            pos2(rect.left() + 44., rect.center().y),
                            tr(name),
                            14.,
                            if active { purple() } else { ink() },
                            Align2::LEFT_CENTER,
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                true,
                                active,
                                tr(name),
                            )
                        });
                        if response.clicked() {
                            side.mode = *mode;
                        }
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
            ui.add_space(18.);
            ui.horizontal(|ui| {
                ui.label(tr("Màu"));
                self.edit_color(ui, "side", &mut side.color);
                ui.checkbox(&mut side.rainbow, "RGB");
            });
            ui.horizontal(|ui| {
                ui.label(tr("Độ sáng"));
                options_slider(ui, egui::Slider::new(&mut side.brightness, 1..=6));
            });
            ui.horizontal(|ui| {
                ui.label(tr("Tốc độ"));
                options_slider(ui, egui::Slider::new(&mut side.speed, 1..=6));
            });
        });
        if !supported {
            ui.label(
                RichText::new(tr("Kết nối để kiểm tra hỗ trợ đèn hai bên."))
                    .size(14.)
                    .color(muted()),
            );
        }
        if before != side {
            self.profile_mut().side_lighting = Some(side);
            self.changed();
        }
        if self.profile().side_lighting.is_some() && ui.add(button("Hoàn tác", false)).clicked() {
            self.profile_mut().side_lighting = None;
            self.changed();
        }
    }
}

const MANUAL_URL: &str = "https://drive.google.com/file/d/12tl04Z6Ol5kfUWCdcbDVPiEMCkWKF_vb/view";
struct LightingShortcut {
    label: &'static str,
    keys: &'static [u16],
    accessible_keys: &'static str,
    note: Option<&'static str>,
}
fn shortcut_rows(zone: u8) -> &'static [LightingShortcut] {
    const POWER: LightingShortcut = LightingShortcut {
        label: "Bật / tắt chế độ tiết kiệm đèn",
        keys: &[175, 229],
        accessible_keys: "Fn + Right Shift",
        note: Some("Tắt đèn nền và đèn hai bên; giữ đèn báo Caps Lock và khoá Win."),
    };
    match zone {
        1 => &[
            LightingShortcut {
                label: "Màu đèn báo cạnh núm",
                keys: &[175, 76],
                accessible_keys: "Fn + Delete",
                note: Some("Chuyển giữa 7 màu đèn báo cạnh núm."),
            },
            LightingShortcut {
                label: "Chế độ đèn báo cạnh núm",
                keys: &[175, 74],
                accessible_keys: "Fn + Home",
                note: Some("Luôn sáng, nhịp thở hoặc tắt đèn báo cạnh núm."),
            },
        ],
        2 => &[
            LightingShortcut {
                label: "Đổi hiệu ứng đèn hai bên",
                keys: &[175, 42],
                accessible_keys: "Fn + Backspace",
                note: None,
            },
            POWER,
        ],
        _ => &[
            LightingShortcut {
                label: "Đổi hiệu ứng đèn nền",
                keys: &[175, 49],
                accessible_keys: "Fn + Backslash",
                note: None,
            },
            LightingShortcut {
                label: "Độ sáng",
                keys: &[175, 82, 81],
                accessible_keys: "Fn + Up / Down",
                note: None,
            },
            LightingShortcut {
                label: "Tốc độ",
                keys: &[175, 80, 79],
                accessible_keys: "Fn + Left / Right",
                note: None,
            },
            POWER,
        ],
    }
}
