use super::*;
impl NutApp {
    pub(super) fn go_home(&mut self) {
        self.stop_recording();
        self.home = true;
        self.selected = None;
        self.function = false;
        self.connect_open = false;
        self.appearance_open = false;
        if !self.busy {
            self.send(Command::Scan);
        }
    }
    pub(super) fn open_device(&mut self, entry: Option<DeviceEntry>) {
        self.home = false;
        self.tab = Tab::Keys;
        self.selected = None;
        self.function = false;
        self.connect_open = false;
        if let Some(entry) = entry
            && !self.busy
            && !self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.entry.path == entry.path)
        {
            self.send(Command::Connect(entry));
        }
    }
    pub(super) fn home_stage(&mut self, ui: &mut egui::Ui, r: Rect) {
        text(
            ui.painter(),
            pos2(40., 38.),
            "Thiết bị của bạn",
            26.,
            ink(),
            Align2::LEFT_CENTER,
        );
        let add = Rect::from_min_size(pos2(r.right() - 276., 20.), vec2(190., 36.));
        if ui.put(add, button("+ Thêm thiết bị", false)).clicked() {
            self.connect_open = true;
            if !self.busy {
                self.send(Command::Scan);
            }
        }
        let settings = Rect::from_min_size(pos2(r.right() - 72., 20.), vec2(36., 36.));
        let response = ui.interact(settings, egui::Id::new("home_settings"), Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Cài đặt"))
        });
        icon(ui.painter(), settings.center(), 4, ink(), 0.9);
        if response.clicked() {
            self.home = false;
            self.tab = Tab::Device;
        }
        let mut known = self.library.devices.clone();
        for entry in self.devices.iter().filter(|e| {
            e.supported
                && !self
                    .library
                    .device_hidden(&e.name, e.vendor_id, e.product_id)
        }) {
            if !known.iter().any(|d| {
                d.name == entry.name
                    && d.product_id == entry.product_id
                    && d.vendor_id == entry.vendor_id
            }) {
                known.push(model::KnownDevice {
                    name: entry.name.clone(),
                    vendor_id: entry.vendor_id,
                    product_id: entry.product_id,
                });
            }
        }
        if known.is_empty() {
            let c = r.center();
            icon(ui.painter(), c - vec2(0., 66.), 0, muted(), 2.0);
            text(
                ui.painter(),
                c - vec2(0., 12.),
                if self.busy {
                    "Đang tìm thiết bị…"
                } else {
                    "Chưa có thiết bị"
                },
                20.,
                ink(),
                Align2::CENTER_CENTER,
            );
            text(
                ui.painter(),
                c + vec2(0., 24.),
                "Kết nối NUT75 bằng USB-C hoặc receiver 2.4 GHz.",
                14.,
                muted(),
                Align2::CENTER_CENTER,
            );
            return;
        }
        let inner = Rect::from_min_max(r.min + vec2(40., 100.), r.max - vec2(40., 32.));
        let cols = known.len().min(if inner.width() > 1000. { 2 } else { 1 });
        let width = ((inner.width() - 32.) / cols as f32).min(600.);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        let mut open = None;
        let mut remove = None;
        egui::ScrollArea::vertical()
            .id_salt("device_home")
            .show(&mut child, |ui| {
                if known.len() <= cols {
                    ui.add_space(((inner.height() - 360.) / 2.).max(24.));
                }
                for row in known.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.add_space(
                            ((inner.width() - row.len() as f32 * (width + 24.) + 24.) / 2.).max(0.),
                        );
                        for device in row {
                            let available = self
                                .devices
                                .iter()
                                .find(|e| {
                                    e.supported
                                        && e.name == device.name
                                        && e.vendor_id == device.vendor_id
                                        && e.product_id == device.product_id
                                })
                                .cloned();
                            let connected = self.snapshot.as_ref().filter(|s| {
                                s.entry.name == device.name
                                    && s.entry.product_id == device.product_id
                                    && s.entry.vendor_id == device.vendor_id
                            });
                            let (rect, response) =
                                ui.allocate_exact_size(vec2(width, 340.), Sense::click());
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    format!("{} {}", tr("Mở"), device.name),
                                )
                            });
                            if response.hovered() {
                                ui.painter().rect_filled(rect, 12., lilac());
                            }
                            let image = Rect::from_center_size(
                                rect.center() - vec2(0., 38.),
                                vec2(width - 28., (width - 28.) * 0.4),
                            );
                            preview::paint(
                                ui.painter(),
                                image,
                                &self.areas,
                                self.library.colorway,
                                None,
                                self.keyboard.id(),
                                &Default::default(),
                            );
                            text(
                                ui.painter(),
                                pos2(rect.center().x, rect.bottom() - 80.),
                                "WEIKAV NUT75",
                                17.,
                                ink(),
                                Align2::CENTER_CENTER,
                            );
                            let connection = if device.product_id == crate::device::PRODUCT_USB {
                                "USB-C"
                            } else {
                                "2.4 GHz"
                            };
                            let status = if connected.is_some() {
                                format!("{}  -  {connection}", tr("Đã kết nối"))
                            } else if available
                                .as_ref()
                                .is_some_and(|e| self.connection_target.as_ref() == Some(&e.path))
                            {
                                tr("Đang kết nối…").into()
                            } else if available.is_some() {
                                format!("{}  -  {connection}", tr("Đã phát hiện thiết bị"))
                            } else {
                                tr("Ngoại tuyến").into()
                            };
                            text(
                                ui.painter(),
                                pos2(rect.center().x, rect.bottom() - 46.),
                                &status,
                                14.,
                                if connected.is_some() {
                                    green()
                                } else {
                                    muted()
                                },
                                Align2::CENTER_CENTER,
                            );
                            if let Some(s) = connected {
                                let percent = if s.entry.product_id != crate::device::PRODUCT_USB
                                    && s.info.battery <= 100
                                {
                                    Some(s.info.battery)
                                } else {
                                    None
                                };
                                let label = match (percent, s.info.charging) {
                                    (Some(n), true) => format!("{n}% - {}", tr("Đang sạc")),
                                    (Some(n), false) => format!("{n}%"),
                                    (None, true) => tr("Đang sạc").to_owned(),
                                    _ => String::new(),
                                };
                                if !label.is_empty() {
                                    let galley = ui.painter().layout_no_wrap(
                                        label.clone(),
                                        FontId::proportional(13.),
                                        green(),
                                    );
                                    let x = rect.center().x - (galley.size().x + 26.) / 2.;
                                    let y = rect.bottom() - 18.;
                                    let battery =
                                        Rect::from_center_size(pos2(x + 9., y), vec2(18., 10.));
                                    ui.painter().rect_stroke(
                                        battery,
                                        2.,
                                        Stroke::new(1.2_f32, green()),
                                        StrokeKind::Inside,
                                    );
                                    ui.painter().rect_filled(
                                        Rect::from_center_size(
                                            battery.right_center() + vec2(1., 0.),
                                            vec2(2., 4.),
                                        ),
                                        1.,
                                        green(),
                                    );
                                    let fill = percent.map(|n| n as f32 / 100.).unwrap_or(0.65);
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(
                                            battery.min + vec2(2., 2.),
                                            vec2(14. * fill, 6.),
                                        ),
                                        1.,
                                        green(),
                                    );
                                    ui.painter().galley(
                                        pos2(x + 26., y - galley.size().y / 2.),
                                        galley,
                                        green(),
                                    );
                                    response.clone().on_hover_text(label);
                                }
                            }
                            let close = Rect::from_center_size(
                                rect.right_top() + vec2(-24., 24.),
                                vec2(32., 32.),
                            );
                            let close_name = format!("{} {}", tr("Xoá thiết bị"), device.name);
                            let close_id = ui.make_persistent_id(&close_name);
                            let reveal = ui.rect_contains_pointer(rect)
                                || response.has_focus()
                                || ui.memory(|m| m.has_focus(close_id));
                            let mut deleting = false;
                            if reveal
                                && !self.busy
                                && icon_button_at(ui, close, 5, &close_name).clicked()
                            {
                                deleting = true;
                            }
                            if deleting {
                                remove = Some((device.clone(), connected.is_some()));
                            } else if response.clicked()
                                && !close
                                    .contains(ui.ctx().pointer_latest_pos().unwrap_or(Pos2::ZERO))
                            {
                                open = Some(available);
                            }

                            response.on_hover_cursor(egui::CursorIcon::PointingHand);
                            ui.add_space(24.);
                        }
                    });
                    ui.add_space(24.);
                }
            });
        if let Some((device, connected)) = remove {
            self.library.forget_device(&device);
            self.changed();
            if connected {
                self.send(Command::Disconnect);
            }
        } else if let Some(entry) = open {
            self.open_device(entry);
        }
    }
}
