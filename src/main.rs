#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod device;
mod features;
mod media_input;
mod model;
mod preview;
mod protocol;

fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--probe-led-colors") {
        if let Err(e) = device::probe_led_colors() {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--test-red-light") {
        if let Err(e) = device::test_red_light(None) {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if let Some(index) = args.iter().position(|a| a == "--restore-light") {
        let result = args
            .get(index + 1)
            .ok_or_else(|| anyhow::anyhow!("A lighting backup path is required"))
            .and_then(|path| device::test_red_light(Some(std::path::Path::new(path))));
        if let Err(e) = result {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if std::env::args().any(|a| a == "--probe-bluetooth") {
        if let Err(e) = media_input::probe_bluetooth() {
            eprintln!("{e:#}");
        }
        return Ok(());
    }
    if std::env::args().any(|a| a == "--watch-input") {
        if let Err(e) = media_input::watch() {
            eprintln!("{e:#}");
        }
        return Ok(());
    }
    if std::env::args().any(|a| a == "--probe-zones") {
        if let Err(e) = device::probe_zones() {
            eprintln!("{e:#}");
        }
        return Ok(());
    }
    if std::env::args().any(|a| a == "--probe-input") {
        if let Err(e) = media_input::probe() {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if std::env::args().any(|arg| arg == "--probe") {
        if let Err(error) = device::probe() {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if std::env::args().any(|arg| arg == "--list-devices") {
        match device::enumerate() {
            Ok(devices) => println!("{}", serde_json::to_string_pretty(&devices).unwrap()),
            Err(error) => eprintln!("{error:#}"),
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Weikav Studio")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/app-icon/icon.png"))
                    .expect("bundled NUT75 White icon"),
            )
            .with_title_shown(false)
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1120.0, 740.0])
            .with_app_id("studio.nut75.desktop"),
        ..Default::default()
    };
    eframe::run_native(
        "Weikav Studio",
        options,
        Box::new(|cc| Ok(Box::new(app::NutApp::new(cc)))),
    )
}
