//! Screen-only, deterministic visual approximations of the device's lighting modes.
//! This module performs no HID operations.
use crate::model::{Colorway, KeyArea, Lighting};
use eframe::egui::{self, Align2, Color32, FontId, Rect, Stroke, StrokeKind, pos2, vec2};

pub struct Ripple {
    pub x: f32,
    pub y: f32,
    pub time: f64,
}

fn rgb(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
fn tone(color: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * factor).min(255.) as u8,
        (color.g() as f32 * factor).min(255.) as u8,
        (color.b() as f32 * factor).min(255.) as u8,
    )
}
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0., 1.);
    Color32::from_rgb(
        (a.r() as f32 * (1. - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1. - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1. - t) + b.b() as f32 * t) as u8,
    )
}

pub fn sample(light: &Lighting, x: f32, y: f32, time: f64, ripples: &[Ripple]) -> (Color32, f32) {
    if light.mode == 0 {
        return (Color32::BLACK, 0.);
    }
    let t = time as f32 * (0.18 + light.speed as f32 * 0.12);
    let (x, y) = match light.direction {
        1 => (1. - x, y),
        2 => (y, x),
        3 => (1. - y, x),
        _ => (x, y),
    };
    let mut hue = t * 0.12 + x * 0.5;
    let mut energy = 1.;
    let radial = ((x - 0.5).powi(2) + ((y - 0.5) * 0.45).powi(2)).sqrt();
    let sweep = |v: f32| 0.06 + 0.94 * v.sin().max(0.).powf(3.);
    let twinkle = ((x * 97. + y * 137.).sin() * 43758.547).fract().abs();
    match light.mode {
        2 | 3 | 13 | 14 | 15 => {
            energy = 0.;
            // A slow demonstration pulse runs until the user presses a key.
            if ripples.is_empty() {
                let front = (t * 0.24).fract() * 1.3;
                energy = (1. - (radial - front).abs() / 0.1).clamp(0., 1.);
            }
            for ripple in ripples {
                let age = (time - ripple.time) as f32;
                let distance = ((x - ripple.x).powi(2) + ((y - ripple.y) * 0.45).powi(2)).sqrt();
                if (0. ..2.4).contains(&age) {
                    let value = if matches!(light.mode, 2 | 3) {
                        (1. - distance / 0.065).clamp(0., 1.) * (1. - age / 2.4)
                    } else if light.mode == 13 {
                        (1. - age / 1.5).max(0.)
                    } else {
                        let radius = age * 0.6;
                        let mut v = (1. - (distance - radius).abs() / 0.1).clamp(0., 1.);
                        if light.mode == 14 {
                            v = v.max((1. - (distance - radius * 0.5).abs() / 0.06).clamp(0., 1.));
                        }
                        v * (1. - age / 2.4)
                    };
                    energy = energy.max(value);
                }
            }
            if light.mode == 3 {
                energy = 1. - energy;
            }
            hue = radial - t * 0.12;
        }
        4 => energy = sweep(t * 2. + twinkle * 15.),
        5 => {
            energy = sweep(y * 8. - t * 3. + twinkle * 8.);
            hue = y * 0.6 + t * 0.1;
        }
        6 => {
            hue = twinkle + t * 0.15;
            energy = 0.3 + 0.7 * (t + twinkle * 9.).sin().abs();
        }
        7 => energy = 0.06 + 0.94 * (0.5 + 0.5 * (t * 2.4).sin()).powf(1.7),
        8 => hue = t * 0.18,
        9 => {
            energy = sweep(radial * 18. - t * 3.);
            hue = radial * 2. - t * 0.15;
        }
        10 => {
            energy = sweep(x * 10. - t * 3.).max(sweep(y * 9. + t * 3.));
            hue = x + y - t * 0.2;
        }
        11 => {
            hue = x * 1.4 - t * 0.22;
            energy = if light.rainbow {
                1.
            } else {
                sweep(x * 7. - t * 2.7)
            };
        }
        12 => {
            hue = (x - 0.5).abs() * 2. - t * 0.2;
            energy = sweep((x - 0.5).abs() * 16. - t * 3. + y * 3.);
        }
        16 => {
            energy = sweep(x * 15. - t * 4. + (y * 12.).sin());
            hue = x * 2. - t * 0.3;
        }
        17 => {
            energy = sweep((x * 6.).sin() * 2. + y * 8. - t * 2.);
            hue = x + y - t * 0.2;
        }
        18 => {
            energy = sweep(x * 12. + y * 6. - t * 4.);
            hue = x + y - t * 0.2;
        }
        19 => {
            let front = 0.5 + 0.5 * (t * 1.8).sin();
            energy = (1. - (x - front).abs() / 0.2).clamp(0., 1.);
            hue = x + t * 0.1;
        }
        23 => hue = (y - 0.5).atan2((x - 0.5) * 2.2) / std::f32::consts::TAU - t * 0.18,
        24 => {
            hue = radial * 2. + t * 0.2;
            energy = sweep(radial * 20. + t * 4.);
        }
        25 => {
            hue = x * 0.5 + t * 0.12;
            energy = sweep(x * 10. - t * 2.).max(sweep(y * 10. + t * 2.));
        }
        _ => {}
    }
    let color = if light.rainbow || matches!(light.mode, 6 | 8 | 23) {
        Color32::from(egui::ecolor::Hsva::new(hue.rem_euclid(1.), 0.86, 1., 1.))
    } else {
        rgb(light.color)
    };
    (color, energy * light.brightness.clamp(1, 6) as f32 / 6.)
}

pub fn paint(
    p: &egui::Painter,
    rect: Rect,
    keys: &[KeyArea],
    colorway: Colorway,
    lighting: Option<(&Lighting, f64, &[Ripple])>,
    original: egui::TextureId,
    custom: &std::collections::BTreeMap<u8, [u8; 3]>,
) {
    let scale = rect.width() / 1000.;
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * scale;
    if colorway == Colorway::IceBlue {
        p.image(
            original,
            rect,
            Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
            Color32::WHITE,
        );
        if let Some((light, time, ripples)) = lighting {
            for key in keys {
                let [x, y, w, h] = key.rect;
                let r = Rect::from_min_size(at(x, y), vec2(w, h) * scale);
                let (mut color, intensity) = sample(
                    light,
                    (x + w / 2. - 91.) / 819.,
                    (y + h / 2. - 40.) / 320.,
                    time,
                    ripples,
                );
                if light.mode == 128 {
                    color = rgb(*custom.get(&key.id).unwrap_or(&[0; 3]));
                }
                for pass in (1..=3).rev() {
                    p.rect_stroke(
                        r.expand(pass as f32 * scale),
                        3. * scale,
                        Stroke::new(2. * scale, color.gamma_multiply(intensity * 0.16)),
                        StrokeKind::Middle,
                    );
                }
                p.rect_stroke(
                    r,
                    3. * scale,
                    Stroke::new(1.7 * scale, color.gamma_multiply(intensity)),
                    StrokeKind::Inside,
                );
                p.rect_filled(
                    r.shrink(scale),
                    3. * scale,
                    color.gamma_multiply(intensity * 0.17),
                );
            }
        }
        return;
    }
    let case = Rect::from_min_max(at(76., 15.), at(924., 385.));
    let case_color = rgb(colorway.case());
    p.rect_filled(
        case.translate(vec2(0., 4. * scale)),
        10. * scale,
        tone(case_color, 0.62),
    );
    p.rect(
        case,
        9. * scale,
        case_color,
        Stroke::new(1.2 * scale, tone(case_color, 0.75)),
        StrokeKind::Inside,
    );
    p.rect_stroke(
        case.shrink(2. * scale),
        7. * scale,
        Stroke::new(1. * scale, tone(case_color, 1.17)),
        StrokeKind::Inside,
    );

    for key in keys {
        let [x, y, w, h] = key.rect;
        let r = Rect::from_min_size(at(x, y), vec2(w, h) * scale);
        if !p.clip_rect().intersects(r.expand(3. * scale)) {
            continue;
        }
        let (cap, legend) = colorway.key_colors(key.id);
        let mut cap = rgb(cap);
        let signal = lighting.map(|(light, t, pulses)| {
            let (mut color, energy) = sample(
                light,
                (x + w / 2. - 91.) / 819.,
                (y + h / 2. - 40.) / 320.,
                t,
                pulses,
            );
            if light.mode == 128 {
                color = rgb(*custom.get(&key.id).unwrap_or(&[0; 3]));
            }
            (color, energy)
        });
        p.rect_filled(r.expand(1.1 * scale), 3. * scale, tone(case_color, 0.25));
        if let Some((color, intensity)) = signal {
            for pass in (1..=3).rev() {
                p.rect_stroke(
                    r.expand(pass as f32 * scale),
                    4. * scale,
                    Stroke::new(2. * scale, color.gamma_multiply(intensity * 0.18)),
                    StrokeKind::Middle,
                );
            }
            p.rect_stroke(
                r.expand(0.8 * scale),
                3. * scale,
                Stroke::new(1.7 * scale, color.gamma_multiply(intensity)),
                StrokeKind::Middle,
            );
            cap = mix(cap, color, intensity * 0.24);
        }
        p.rect_filled(r, 3. * scale, tone(cap, 0.76));
        let face = Rect::from_min_max(r.min + vec2(4., 3.) * scale, r.max - vec2(4., 7.) * scale);
        p.rect_filled(face, 2.5 * scale, cap);
        p.line_segment(
            [
                face.left_top() + vec2(2., 0.),
                face.right_top() - vec2(2., 0.),
            ],
            Stroke::new(scale, tone(cap, 1.12)),
        );
        let label = match key.id {
            16 => "~",
            17..=26 => key.key.name.split_whitespace().next().unwrap_or(""),
            27 => "−",
            28 => "=",
            43 => "[",
            44 => "]",
            58 => ";",
            59 => "'",
            60 => "\\",
            72 => ",",
            73 => ".",
            74 => "/",
            64 | 75 => "Shift",
            80 | 87 => "Ctrl",
            81 => "Win",
            82 => "Alt",
            83 => "",
            _ => &key.key.name,
        };
        let size = if label.chars().count() > 4 {
            9.5
        } else if label.chars().count() == 1 {
            15.
        } else {
            10.5
        };
        p.text(
            face.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(size * scale),
            rgb(legend),
        );
    }
    let roller = Rect::from_min_max(at(875., 273.), at(913., 295.));
    p.rect_filled(
        roller.expand(2. * scale),
        3. * scale,
        tone(case_color, 0.65),
    );
    p.rect_filled(roller, 3. * scale, Color32::from_gray(63));
    // Keep the original metal wheel and its photographic texture on every colorway.
    p.image(
        original,
        Rect::from_min_max(at(879., 275.), at(911., 293.)),
        Rect::from_min_max(
            pos2(879. / 1000., 275. / 400.),
            pos2(911. / 1000., 293. / 400.),
        ),
        Color32::WHITE,
    );
}

/// Separate auxiliary lighting; never imply that the main RGB effect controls side LEDs.
pub fn paint_sides(p: &egui::Painter, rect: Rect, light: &Lighting, time: f64) {
    let scale = rect.width() / 1000.;
    for x in [80., 918.] {
        for i in 0..24 {
            let (color, intensity) = sample(
                light,
                if x < 100. { 0. } else { 1. },
                i as f32 / 24.,
                time,
                &[],
            );
            let r = Rect::from_min_size(
                rect.min + vec2(x, 100. + i as f32 * 8.) * scale,
                vec2(2., 7.) * scale,
            );
            p.rect_filled(
                r.expand(3. * scale),
                3. * scale,
                color.gamma_multiply(intensity * 0.15),
            );
            p.rect_filled(r, scale, color.gamma_multiply(intensity * 0.85));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_official_effects_have_bounded_previews_and_reactive_keys_respond() {
        for (mode, _) in crate::model::LIGHT_MODES {
            let light = Lighting {
                mode: *mode,
                ..Default::default()
            };
            for time in [0., 0.25, 1., 4., 20.] {
                for x in [0., 0.5, 1.] {
                    let (_, energy) = sample(&light, x, 0.5, time, &[]);
                    assert!(energy.is_finite() && (0. ..=1.).contains(&energy));
                }
            }
        }
        let light = Lighting {
            mode: 2,
            ..Default::default()
        };
        let pulse = [Ripple {
            x: 0.3,
            y: 0.4,
            time: 1.,
        }];
        assert!(sample(&light, 0.3, 0.4, 1.1, &pulse).1 > sample(&light, 0.8, 0.4, 1.1, &pulse).1);
    }
    #[test]
    fn animation_changes_with_time_and_wave_position_but_static_does_not() {
        let mut l = Lighting::default();
        assert_eq!(sample(&l, 0., 0., 0., &[]), sample(&l, 1., 1., 5., &[]));
        l.mode = 7;
        assert_ne!(sample(&l, 0., 0., 0., &[]), sample(&l, 0., 0., 1., &[]));
        l.mode = 11;
        l.rainbow = true;
        assert_ne!(sample(&l, 0., 0., 0., &[]), sample(&l, 1., 0., 0., &[]));
        l.mode = 0;
        assert_eq!(sample(&l, 0., 0., 0., &[]).1, 0.);
    }
}
