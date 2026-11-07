use super::*;
use egui::ecolor::Hsva;

#[derive(Default)]
pub(super) struct ColorPicker {
    owner: Option<egui::Id>,
    hsv: Hsva,
    last: [u8; 3],
    hex: String,
}
impl ColorPicker {
    fn set_rgb(&mut self, color: [u8; 3]) {
        let mut hsv = Hsva::from_srgb(color);
        if hsv.s == 0. {
            hsv.h = self.hsv.h;
        }
        self.hsv = hsv;
        self.last = color;
        self.hex = hex(color);
    }
    fn update_hsv(&mut self, color: &mut [u8; 3]) {
        *color = self.hsv.to_srgb();
        self.last = *color;
        self.hex = hex(*color);
    }
}
fn hex(c: [u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}
fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}
fn wheel_hs(offset: Vec2, radius: f32) -> (f32, f32) {
    (
        ((offset.y.atan2(offset.x) + std::f32::consts::PI) / std::f32::consts::TAU).rem_euclid(1.),
        (offset.length() / radius).clamp(0., 1.),
    )
}
fn marker(p: &egui::Painter, center: Pos2, r: f32) {
    p.circle_stroke(
        center,
        r + 1.,
        Stroke::new(1.0_f32, Color32::from_black_alpha(95)),
    );
    p.circle_stroke(center, r, Stroke::new(2.0_f32, Color32::WHITE));
}
fn wheel(p: &egui::Painter, center: Pos2, radius: f32) {
    const STEPS: usize = 96;
    const RINGS: usize = 24;
    let mut mesh = egui::Mesh::default();
    for ring in 0..=RINGS {
        let saturation = ring as f32 / RINGS as f32;
        for step in 0..=STEPS {
            let h = step as f32 / STEPS as f32;
            let angle = h * std::f32::consts::TAU - std::f32::consts::PI;
            mesh.colored_vertex(
                center + Vec2::angled(angle) * radius * saturation,
                Color32::from_rgb(
                    Hsva::new(h, saturation, 1., 1.).to_srgb()[0],
                    Hsva::new(h, saturation, 1., 1.).to_srgb()[1],
                    Hsva::new(h, saturation, 1., 1.).to_srgb()[2],
                ),
            );
        }
    }
    for ring in 0..RINGS {
        for step in 0..STEPS {
            let a = (ring * (STEPS + 1) + step) as u32;
            let b = a + (STEPS + 1) as u32;
            mesh.add_triangle(a, b, a + 1);
            mesh.add_triangle(a + 1, b, b + 1);
        }
    }
    p.add(egui::Shape::mesh(mesh));
}
const PRESETS: [[u8; 3]; 14] = [
    [255, 79, 84],
    [245, 64, 221],
    [174, 65, 222],
    [137, 99, 250],
    [72, 65, 244],
    [76, 139, 255],
    [26, 199, 204],
    [31, 224, 190],
    [101, 221, 43],
    [204, 231, 0],
    [255, 213, 0],
    [255, 162, 62],
    [255, 255, 255],
    [153, 153, 153],
];

impl NutApp {
    pub(super) fn edit_color(&mut self, ui: &mut egui::Ui, key: &str, color: &mut [u8; 3]) -> bool {
        let (changed, saved) = picker_button(
            ui,
            key,
            color,
            &mut self.color_picker,
            &mut self.library.color_palette,
        );
        if saved {
            self.changed();
        }
        changed
    }
}
fn picker_button(
    ui: &mut egui::Ui,
    key: &str,
    color: &mut [u8; 3],
    state: &mut ColorPicker,
    palette: &mut Vec<[u8; 3]>,
) -> (bool, bool) {
    let before = *color;
    let mut saved = false;
    let (r, response) = ui.allocate_exact_size(vec2(32., 32.), Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Chọn màu")));
    ui.painter().circle_filled(
        r.center(),
        13.,
        Color32::from_rgb(color[0], color[1], color[2]),
    );
    ui.painter()
        .circle_stroke(r.center(), 13., Stroke::new(1.0_f32, line()));
    if response.hovered() {
        ui.painter().circle_stroke(
            r.center(),
            15.,
            Stroke::new(1.0_f32, purple().gamma_multiply(0.5)),
        );
    }
    let popup_id = ui.make_persistent_id(("color_picker", key));
    if response.clicked() || state.owner != Some(popup_id) {
        state.owner = Some(popup_id);
        state.set_rgb(*color);
    }
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .width(304.)
        .frame(
            egui::Frame::popup(ui.style())
                .fill(surface())
                .stroke(Stroke::new(1.0_f32, line()))
                .corner_radius(10.)
                .inner_margin(16),
        )
        .show(|ui| {
            ui.spacing_mut().interact_size.y = 28.;
            ui.spacing_mut().button_padding = vec2(4., 4.);
            ui.set_width(272.);
            if state.last != *color {
                state.set_rgb(*color);
            }
            let (canvas, _) = ui.allocate_exact_size(vec2(272., 438.), Sense::hover());
            let origin = canvas.min;
            text(
                ui.painter(),
                origin + vec2(0., 10.),
                tr("Màu"),
                16.,
                ink(),
                Align2::LEFT_CENTER,
            );
            if icon_button_at(
                ui,
                Rect::from_center_size(origin + vec2(264., 8.), vec2(26., 26.)),
                5,
                tr("Đóng bảng"),
            )
            .clicked()
            {
                ui.close();
            }
            let center = origin + vec2(136., 122.);
            let radius = 94.;
            wheel(ui.painter(), center, radius);
            let response = ui.interact(
                Rect::from_center_size(center, vec2(radius * 2., radius * 2.)),
                popup_id.with("wheel"),
                Sense::click_and_drag(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Slider,
                    true,
                    tr("Sắc độ và độ bão hoà"),
                )
            });
            if (response.clicked() || response.dragged())
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let (h, s) = wheel_hs(pointer - center, radius);
                state.hsv.h = h;
                state.hsv.s = s;
                state.update_hsv(color);
            }
            let angle = state.hsv.h * std::f32::consts::TAU - std::f32::consts::PI;
            marker(
                ui.painter(),
                center + Vec2::angled(angle) * radius * state.hsv.s,
                7.,
            );
            let bar = Rect::from_min_size(origin + vec2(0., 234.), vec2(272., 18.));
            let full = Hsva::new(state.hsv.h, state.hsv.s, 1., 1.).to_srgb();
            let full = Color32::from_rgb(full[0], full[1], full[2]);
            let gradient = Rect::from_min_max(bar.min + vec2(9., 0.), bar.max - vec2(9., 0.));
            let mut mesh = egui::Mesh::default();
            for (point, color) in [
                (gradient.left_top(), Color32::BLACK),
                (gradient.right_top(), full),
                (gradient.left_bottom(), Color32::BLACK),
                (gradient.right_bottom(), full),
            ] {
                mesh.colored_vertex(point, color);
            }
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(1, 3, 2);
            ui.painter().add(egui::Shape::mesh(mesh));
            ui.painter()
                .circle_filled(bar.left_center() + vec2(9., 0.), 9., Color32::BLACK);
            ui.painter()
                .circle_filled(bar.right_center() - vec2(9., 0.), 9., full);
            let response = ui.interact(
                bar.expand(4.),
                popup_id.with("value"),
                Sense::click_and_drag(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, tr("Độ sáng màu"))
            });
            if (response.clicked() || response.dragged())
                && let Some(pointer) = response.interact_pointer_pos()
            {
                state.hsv.v = ((pointer.x - bar.left() - 9.) / (bar.width() - 18.)).clamp(0., 1.);
                state.update_hsv(color);
            }
            marker(
                ui.painter(),
                pos2(
                    bar.left() + 9. + state.hsv.v * (bar.width() - 18.),
                    bar.center().y,
                ),
                7.,
            );
            let mut rgb = *color;
            for i in 0..3 {
                let r = Rect::from_min_size(origin + vec2(i as f32 * 54., 264.), vec2(48., 28.));
                ui.put(
                    r,
                    egui::DragValue::new(&mut rgb[i])
                        .range(0..=255)
                        .speed(1.)
                        .prefix(["R ", "G ", "B "][i]),
                );
            }
            if rgb != *color {
                *color = rgb;
                state.set_rgb(rgb);
            }
            let response = ui.put(
                Rect::from_min_size(origin + vec2(164., 264.), vec2(108., 28.)),
                egui::TextEdit::singleline(&mut state.hex)
                    .font(FontId::monospace(14.))
                    .char_limit(7)
                    .margin(vec2(6., 5.))
                    .hint_text("HEX"),
            );
            if response.changed()
                && let Some(value) = parse_hex(&state.hex)
            {
                *color = value;
                state.set_rgb(value);
            }
            if response.lost_focus() && parse_hex(&state.hex).is_none() {
                state.hex = hex(*color);
            }
            for (i, value) in PRESETS.iter().enumerate() {
                let center = origin + vec2(16. + (i % 7) as f32 * 40., 320. + (i / 7) as f32 * 34.);
                if swatch(
                    ui,
                    popup_id.with(("preset", i)),
                    center,
                    *value,
                    *color == *value,
                )
                .clicked()
                {
                    *color = *value;
                    state.set_rgb(*value);
                }
            }
            text(
                ui.painter(),
                origin + vec2(0., 388.),
                tr("Màu riêng"),
                13.,
                muted(),
                Align2::LEFT_CENTER,
            );
            let add = origin + vec2(16., 422.);
            let r = Rect::from_center_size(add, vec2(28., 28.));
            let response = ui.interact(r, popup_id.with("save"), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Lưu màu hiện tại"))
            });
            ui.painter().circle_filled(add, 12., shade(230));
            for v in [vec2(4., 0.), vec2(0., 4.)] {
                ui.painter()
                    .line_segment([add - v, add + v], Stroke::new(1.6_f32, muted()));
            }
            if response.on_hover_text(tr("Lưu màu hiện tại")).clicked() && !palette.contains(color)
            {
                if palette.len() >= 6 {
                    palette.remove(0);
                }
                palette.push(*color);
                saved = true;
            }
            let mut remove = None;
            for i in 0..6 {
                let center = origin + vec2(56. + i as f32 * 40., 422.);
                if let Some(value) = palette.get(i).copied() {
                    let response = swatch(
                        ui,
                        popup_id.with(("saved", i)),
                        center,
                        value,
                        value == *color,
                    );
                    if response.clicked() {
                        *color = value;
                        state.set_rgb(value);
                    }
                    if response.secondary_clicked() {
                        remove = Some(i);
                    }
                    response.on_hover_text(tr("Chuột phải để xoá màu"));
                } else {
                    ui.painter().circle_filled(center, 12., shade(240));
                }
            }
            if let Some(i) = remove {
                palette.remove(i);
                saved = true;
            }
        });
    (before != *color, saved)
}
fn swatch(
    ui: &mut egui::Ui,
    id: egui::Id,
    center: Pos2,
    value: [u8; 3],
    selected: bool,
) -> egui::Response {
    let r = Rect::from_center_size(center, vec2(28., 28.));
    let response = ui.interact(r, id, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            true,
            selected,
            format!("#{}", hex(value)),
        )
    });
    ui.painter()
        .circle_filled(center, 11., Color32::from_rgb(value[0], value[1], value[2]));
    ui.painter()
        .circle_stroke(center, 11., Stroke::new(0.7_f32, line()));
    if selected {
        ui.painter()
            .circle_stroke(center, 14., Stroke::new(1.5_f32, purple()));
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_input_rejects_partial_and_invalid_values_and_round_trips_rgb() {
        for c in [[0, 0, 0], [255, 255, 255], [12, 128, 249]] {
            assert_eq!(parse_hex(&hex(c)), Some(c));
        }
        assert_eq!(parse_hex("#00Ff80"), Some([0, 255, 128]));
        for bad in ["fff", "#00ff", "zz00ff", "ééé", "00000000"] {
            assert_eq!(parse_hex(bad), None);
        }
    }
    #[test]
    fn wheel_maps_reference_hues_and_keeps_hue_when_value_reaches_black() {
        let (h, s) = wheel_hs(vec2(-94., 0.), 94.);
        assert!(h.min(1. - h) < 0.001 && s == 1.);
        let (h, s) = wheel_hs(Vec2::angled(std::f32::consts::PI / 3.) * 94., 94.);
        assert!((h - 2. / 3.).abs() < 0.001 && (s - 1.).abs() < 0.001);
        let mut picker = ColorPicker::default();
        picker.set_rgb([0, 0, 255]);
        picker.hsv.v = 0.;
        let mut c = [0, 0, 0];
        picker.update_hsv(&mut c);
        assert_eq!(c, [0, 0, 0]);
        picker.hsv.v = 1.;
        picker.update_hsv(&mut c);
        assert_eq!(c, [0, 0, 255]);
    }
}
