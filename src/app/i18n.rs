//! Bundled translations; changing language never sends text or device data over the network.
use std::{collections::BTreeMap, sync::OnceLock};
type Catalog = BTreeMap<String, [String; 6]>;
fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/locales/catalog.json"))
            .expect("bundled locale catalog")
    })
}
pub(super) fn lookup(text: &str, language: u8) -> Option<&'static str> {
    let index = language.checked_sub(2)? as usize;
    catalog().get(text)?.get(index).map(String::as_str)
}
pub(super) fn translate(text: &str, language: u8) -> &str {
    lookup(text, language).unwrap_or(text)
}
pub(super) fn message(text: &str, language: u8) -> String {
    if language < 2 {
        return text.to_owned();
    }
    if let Some(translated) = lookup(text, language) {
        return translated.to_owned();
    }
    if text.contains('\n') {
        return text
            .lines()
            .map(|line| message(line, language))
            .collect::<Vec<_>>()
            .join("\n");
    }
    if let Some((context, detail)) = text.split_once(": ")
        && let Some(context) = lookup(context, language)
    {
        return format!("{context}: {}", message(detail, language));
    }
    // Only translate known leading messages, preserving dynamic commands, paths and values.
    let prefix = catalog()
        .keys()
        .filter(|key| text.starts_with(key.as_str()) && text.len() > key.len())
        .filter(|key| {
            text[key.len()..].starts_with(|c: char| c.is_whitespace() || c == '.' || c == ':')
        })
        .max_by_key(|key| key.len());
    if let Some(prefix) = prefix {
        let rest = &text[prefix.len()..];
        let whitespace = rest.len() - rest.trim_start().len();
        return format!(
            "{}{}{}",
            translate(prefix, language),
            &rest[..whitespace],
            message(rest.trim_start(), language)
        );
    }
    text.to_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composed_errors_preserve_user_paths_and_protocol_values_in_every_locale() {
        for language in 2..=7 {
            let text = message("Profile exported: /Users/lit/Hồ sơ/Phím.json", language);
            assert!(text.ends_with("/Users/lit/Hồ sơ/Phím.json"));
            assert!(!text.starts_with("Profile exported"));
            let text = message("The keyboard did not respond to command 0x13", language);
            assert!(text.ends_with(" 0x13"));
            assert!(!text.starts_with("The keyboard"));
        }
    }
    #[test]
    fn bundled_font_covers_every_cjk_character_in_the_catalog() {
        let ctx = eframe::egui::Context::default();
        let mut fonts = eframe::egui::FontDefinitions::default();
        fonts.font_data.insert(
            "CJK".into(),
            eframe::egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/NotoSansCJKsc-Regular.otf"
            ))
            .into(),
        );
        fonts
            .families
            .get_mut(&eframe::egui::FontFamily::Proportional)
            .unwrap()
            .push("CJK".into());
        ctx.set_fonts(fonts);
        let _ = ctx.run(Default::default(), |ctx| {
            ctx.fonts_mut(|fonts| {
                for text in catalog().values().flat_map(|row| row.iter().take(3)) {
                    for c in text
                        .chars()
                        .filter(|c| ('\u{2E80}'..='\u{D7AF}').contains(c))
                    {
                        assert!(
                            fonts.has_glyph(&eframe::egui::FontId::proportional(14.), c),
                            "missing glyph {c}"
                        );
                    }
                }
            });
        });
    }
    #[test]
    fn each_language_is_persisted_without_altering_profile_names() {
        for language in crate::model::Language::ALL {
            let mut library = crate::model::Library {
                language,
                ..Default::default()
            };
            library.profiles[0].name = "My custom hồ sơ".into();
            let restored: crate::model::Library =
                serde_json::from_str(&serde_json::to_string(&library).unwrap()).unwrap();
            assert!(restored.language == language);
            assert_eq!(restored.profiles[0].name, "My custom hồ sơ");
        }
    }
}
