//! NUT75 lighting animation data, in the vendor's physical key order.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

const KEYS: usize = 81;

#[derive(Clone, Deserialize)]
pub struct Animation {
    pub name: String,
    pub speed: u8,
    pub data: Vec<String>,
}

impl Animation {
    pub fn frames(&self) -> Result<Vec<Vec<[u8; 3]>>> {
        ensure!(
            !self.data.is_empty() && self.data.len().is_multiple_of(KEYS),
            "Invalid animation frame length"
        );
        ensure!(self.speed <= 104, "Invalid animation speed");
        self.data
            .chunks(KEYS)
            .map(|frame| {
                frame
                    .iter()
                    .map(|hex| {
                        let s = hex.strip_prefix('#').context("Invalid color")?;
                        let s = match s.len() {
                            3 => s.chars().flat_map(|c| [c, c]).collect::<String>(),
                            6 => s.to_owned(),
                            _ => anyhow::bail!("Invalid color"),
                        };
                        let n = u32::from_str_radix(&s, 16).context("Invalid color")?;
                        Ok([(n >> 16) as u8, (n >> 8) as u8, n as u8])
                    })
                    .collect()
            })
            .collect()
    }
}

pub fn presets() -> Vec<Animation> {
    [
        include_str!("../assets/gif-lighting/heart_81_75.json"),
        include_str!("../assets/gif-lighting/glacier_81.json"),
        include_str!("../assets/gif-lighting/spring_81.json"),
        include_str!("../assets/gif-lighting/jade_81.json"),
        include_str!("../assets/gif-lighting/white_81.json"),
        include_str!("../assets/gif-lighting/plum_81.json"),
        include_str!("../assets/gif-lighting/orange_81.json"),
    ]
    .iter()
    .map(|raw| serde_json::from_str(raw).expect("valid vendor animation"))
    .collect()
}

pub fn encode(animation: &Animation, max_frames: u16) -> Result<Vec<u8>> {
    let frames = animation.frames()?;
    ensure!(
        max_frames > 0 && frames.len() <= usize::from(max_frames),
        "Animation exceeds the device frame limit"
    );
    let mut result = Vec::with_capacity(4 + frames.len() * KEYS * 3);
    result.extend_from_slice(&(frames.len() as u16).to_le_bytes());
    result.extend_from_slice(&(106_u16 - u16::from(animation.speed)).to_le_bytes());
    for frame in frames {
        for rgb in frame {
            result.extend_from_slice(&rgb);
        }
    }
    Ok(result)
}

/// Local generative presets, sampled in the manufacturer's 81-key order.
pub fn live_colors(effect: usize, time: f64) -> Vec<[u8; 3]> {
    use eframe::egui::{Color32, ecolor::Hsva};
    crate::model::key_areas()
        .into_iter()
        .map(|area| {
            let [x, y, w, h] = area.rect;
            let x = (x + w * 0.5 - 91.) / 820.;
            let y = (y + h * 0.5 - 40.) / 340.;
            let t = time as f32;
            let (hue, value) = match effect {
                1 => {
                    let d = ((x - 0.5).powi(2) + (y - 0.48).powi(2)).sqrt();
                    let wave = (1. - ((d * 3. - t).rem_euclid(2.) - 1.).abs()).max(0.);
                    ((t * 0.11 + d).rem_euclid(1.), wave.powi(3))
                }
                2 => {
                    let dx = (x - 0.5) * 2.;
                    let dy = (0.5 - y) * 2.;
                    let heart = (dx * dx + dy * dy - 0.55).powi(3) - dx * dx * dy.powi(3);
                    let pulse = 0.45 + 0.55 * (t * 2.).sin().abs();
                    (0.94, if heart < 0. { pulse } else { 0.03 })
                }
                _ => ((x * 0.75 + y * 0.15 + t * 0.1).rem_euclid(1.), 0.9),
            };
            let c: Color32 = Hsva::new(hue, 0.9, value, 1.).into();
            [c.r(), c.g(), c.b()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vendor_presets_validate_and_respect_frame_limits() {
        for preset in presets() {
            let frames = preset.frames().unwrap();
            let bytes = encode(&preset, frames.len() as u16).unwrap();
            assert_eq!(bytes.len(), 4 + frames.len() * KEYS * 3);
            assert_eq!(
                u16::from_le_bytes([bytes[0], bytes[1]]),
                frames.len() as u16
            );
            assert!(encode(&preset, 0).is_err());
        }
    }

    #[test]
    fn live_effect_colors_follow_the_vendor_key_order() {
        let layout: Vec<_> = crate::model::layout()
            .into_iter()
            .flatten()
            .map(|k| k.value)
            .collect();
        let areas: Vec<_> = crate::model::key_areas()
            .into_iter()
            .map(|k| k.id)
            .collect();
        assert_eq!(areas, layout);
        for effect in 0..3 {
            assert_eq!(live_colors(effect, 0.).len(), KEYS);
            assert_ne!(live_colors(effect, 0.), live_colors(effect, 1.));
        }
    }
}
