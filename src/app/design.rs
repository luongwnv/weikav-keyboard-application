use super::*;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
static DARK: AtomicBool = AtomicBool::new(false);
static LANGUAGE: AtomicU8 = AtomicU8::new(0);
pub(super) fn ink() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(236, 237, 242)
    } else {
        Color32::from_rgb(30, 32, 38)
    }
}
pub(super) fn muted() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(163, 165, 176)
    } else {
        Color32::from_rgb(117, 120, 132)
    }
}
pub(super) fn brand() -> Color32 {
    Color32::from_rgb(126, 70, 244)
}
pub(super) fn purple() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(181, 151, 255)
    } else {
        brand()
    }
}
pub(super) fn lilac() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(51, 37, 76)
    } else {
        Color32::from_rgb(242, 236, 255)
    }
}
pub(super) fn line() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(55, 56, 65)
    } else {
        Color32::from_rgb(235, 235, 240)
    }
}
pub(super) fn bg() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(24, 25, 29)
    } else {
        Color32::from_rgb(250, 250, 250)
    }
}
pub(super) fn surface() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(33, 34, 40)
    } else {
        Color32::WHITE
    }
}
pub(super) fn green() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        Color32::from_rgb(75, 196, 143)
    } else {
        Color32::from_rgb(39, 149, 104)
    }
}
pub(super) fn shade(value: u8) -> Color32 {
    Color32::from_gray(if DARK.load(Ordering::Relaxed) {
        255 - value + 18
    } else {
        value
    })
}
pub(super) fn apply(ctx: &egui::Context, lib: &Library) {
    let dark = match lib.appearance {
        model::Appearance::Dark => true,
        model::Appearance::Light => false,
        model::Appearance::System => ctx.system_theme() == Some(egui::Theme::Dark),
    };
    LANGUAGE.store(lib.language as u8, Ordering::Relaxed);
    let changed = DARK.swap(dark, Ordering::Relaxed) != dark;
    if changed
        || ctx.style().visuals.dark_mode != dark
        || ctx.style().visuals.selection.bg_fill != lilac()
    {
        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = bg();
        visuals.window_fill = surface();
        visuals.override_text_color = Some(ink());
        visuals.selection.bg_fill = lilac();
        visuals.selection.stroke = Stroke::new(1.0_f32, purple());
        visuals.widgets.open.bg_fill = surface();
        visuals.widgets.open.weak_bg_fill = surface();
        visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, purple().gamma_multiply(0.45));
        visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, purple());
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = 4.into();
            widget.bg_stroke = Stroke::new(1.0_f32, line());
            widget.expansion = 0.;
        }
        visuals.widgets.inactive.bg_fill = shade(229);
        visuals.widgets.inactive.weak_bg_fill = surface();
        visuals.widgets.hovered.bg_fill = lilac();
        visuals.widgets.hovered.weak_bg_fill = lilac();
        visuals.widgets.active.bg_fill = brand();
        visuals.widgets.active.weak_bg_fill = lilac();
        ctx.set_visuals(visuals);
    }
}
pub(super) fn tr(s: &str) -> &str {
    let language = LANGUAGE.load(Ordering::Relaxed);
    super::i18n::translate(translated(s, language != 0), language)
}
fn translated(s: &str, english: bool) -> &str {
    if !english {
        return s;
    }
    TRANSLATIONS
        .iter()
        .find_map(|(vi, en)| (*vi == s).then_some(*en))
        .unwrap_or(s)
}
// The initial built-in profile label follows the UI language without renaming stored data.
pub(super) fn profile_display_name(name: &str) -> &str {
    if name == "Mặc định" {
        tr(name)
    } else {
        name
    }
}
pub(super) fn localize_message(s: &str) -> String {
    let language = LANGUAGE.load(Ordering::Relaxed);
    if language == 0 {
        s.to_owned()
    } else {
        super::i18n::message(&translated_message(s, true), language)
    }
}
fn translated_message(s: &str, english: bool) -> String {
    if !english {
        return s.to_owned();
    }
    let direct = translated(s, true);
    if direct != s {
        return direct.to_owned();
    }
    if s.contains('\n') {
        return s
            .lines()
            .map(|line| translated_message(line, true))
            .collect::<Vec<_>>()
            .join("\n");
    }
    for suffix in [
        ". Bạn có thể xuất hồ sơ ra tệp JSON.",
        ". Tệp gốc được giữ nguyên; xuất hồ sơ mới để lưu.",
    ] {
        if let Some(prefix) = s.strip_suffix(suffix) {
            return format!(
                "{}. {}",
                translated_message(prefix, true),
                translated(&suffix[2..], true)
            );
        }
    }
    if let Some((context, detail)) = s.split_once(": ") {
        let en = translated(context, true);
        if en != context {
            return format!("{en}: {}", translated_message(detail, true));
        }
    }
    if s.starts_with("Bàn phím không trả lời lệnh ") {
        return s
            .replacen(
                "Bàn phím không trả lời lệnh",
                "The keyboard did not respond to command",
                1,
            )
            .replacen(" tại byte ", " at byte ", 1)
            .replacen(
                "Kiểm tra cáp USB và đóng driver web.",
                "Check the USB cable and close the web driver.",
                1,
            );
    }
    if let Some(bytes) = s.strip_prefix("HID chỉ ghi được ") {
        return format!("HID only wrote {bytes}");
    }
    s.to_owned()
}

const TRANSLATIONS: &[(&str, &str)] = &[
    ("Hồ sơ thiết bị", "Device profile"),
    ("Chế độ", "Mode"),
    ("Hướng dẫn WEIKAV", "WEIKAV guide"),
    (
        "Bật / tắt chế độ tiết kiệm đèn",
        "Toggle lighting power saving",
    ),
    (
        "Tắt đèn nền và đèn hai bên; giữ đèn báo Caps Lock và khoá Win.",
        "Turn off key and side lights; keep Caps Lock and Win-lock indicators.",
    ),
    ("Màu đèn báo cạnh núm", "Roller indicator color"),
    ("Chế độ đèn báo cạnh núm", "Roller indicator mode"),
    (
        "Chuyển giữa 7 màu đèn báo cạnh núm.",
        "Cycle through 7 roller-indicator colors.",
    ),
    (
        "Luôn sáng, nhịp thở hoặc tắt đèn báo cạnh núm.",
        "Roller indicator: always on, breathing or off.",
    ),
    ("Đổi hiệu ứng đèn nền", "Cycle key backlight effects"),
    ("Xoá thiết bị", "Remove device"),
    ("Kết nối và phản hồi", "Connection and response"),
    ("Chức năng phím", "Key behavior"),
    (
        "Hiệu ứng này chạy màu RGB tự động.",
        "This effect uses automatic RGB colors.",
    ),
    ("NÂNG CAO", "ADVANCED"),
    ("Phím nâng cao", "Advanced Keys"),
    ("Phím đã gán", "Current bindings"),
    (
        "Chọn một phím trên bàn phím để gán chức năng.",
        "Select a key on the keyboard to assign an action.",
    ),
    (
        "Chạm một chức năng, giữ một chức năng khác.",
        "Tap for one action, hold for another.",
    ),
    (
        "Nhấn để bật hoặc tắt giữ phím.",
        "Press to toggle holding a key.",
    ),
    (
        "Chọn cách ưu tiên giữa hai phím đối hướng.",
        "Choose the priority between two opposing keys.",
    ),
    (
        "Gán phím cùng các phím bổ trợ.",
        "Assign a key with modifier keys.",
    ),
    ("Xoá", "Delete"),
    ("Di chuyển lên", "Move up"),
    ("Di chuyển xuống", "Move down"),
    (
        "Cross weave trên firmware này chỉ chạy RGB.",
        "Cross weave on this firmware runs in RGB colors.",
    ),
    (
        "Không mở được NUT75. Đóng driver web rồi kết nối lại; trên macOS kiểm tra quyền Giám sát đầu vào nếu hệ thống yêu cầu.",
        "Could not open NUT75. Close the web driver and reconnect. On macOS, check Input Monitoring permissions if prompted.",
    ),
    (
        "Đọc lại keymap không khớp. Có thể một phần cấu hình đã thay đổi; kết nối lại để kiểm tra.",
        "Keymap verification failed. Some settings may have changed. Reconnect to check the keyboard.",
    ),
    (
        "Firmware chưa trả về cấu hình đèn hai bên; chưa thể áp dụng vùng đèn này.",
        "The firmware has not returned side-light settings. This zone cannot be applied yet.",
    ),
    (
        "Thiết bị chưa được xác minh là NUT75. Dùng cáp USB để kết nối.",
        "This device has not been verified as NUT75. Connect with a USB cable.",
    ),
    (
        "Firmware quản lý đèn báo; chưa có lệnh đổi màu được xác minh.",
        "The indicator is managed by firmware; color control is not verified.",
    ),
    (
        "Màu riêng dùng chế độ Tự chọn từng phím cho toàn bàn phím.",
        "A custom color switches the whole keyboard to Per-key lighting.",
    ),
    (
        "Firmware đang ở trạng thái phục hồi; không chỉnh cấu hình",
        "The firmware is in recovery mode. Configuration is unavailable.",
    ),
    (
        "Chưa tìm thấy NUT75. Kết nối USB-C và chọn chế độ USB.",
        "NUT75 was not found. Connect USB-C and select USB mode.",
    ),
    (
        "Thiết bị không có output report 32/64 byte với ID 0",
        "The device has no 32/64-byte output report with ID 0",
    ),
    (
        "Luồng kết nối HID đã dừng. Khởi động lại ứng dụng.",
        "The HID connection stopped. Restart the app.",
    ),
    (
        "Chưa đọc được cấu hình đèn riêng từ firmware này.",
        "This firmware has not returned a usable side-light configuration.",
    ),
    (
        "Chưa hỗ trợ chỉnh đèn hai bên trên firmware này.",
        "Side-light control is not supported on this firmware yet.",
    ),
    (
        "Nếu không đổi đèn, cần đối chiếu thêm qua USB-C.",
        "If the lights do not change, further checks need USB-C.",
    ),
    (
        "Đọc lại RGB không khớp. Kết nối lại để kiểm tra.",
        "Lighting verification failed. Reconnect to check the keyboard.",
    ),
    (
        "Kết nối NUT75 bằng USB-C hoặc receiver 2.4 GHz.",
        "Connect your NUT75 using USB-C or its 2.4 GHz receiver.",
    ),
    (
        "Tệp gốc được giữ nguyên; xuất hồ sơ mới để lưu.",
        "The original file has been kept. Export a new profile to save.",
    ),
    (
        "Cắm USB hoặc đánh thức bàn phím để kết nối.",
        "Connect USB or wake your keyboard.",
    ),
    (
        "Chọn hai phím khác nhau và có thể chỉnh sửa",
        "Choose two different, editable keys",
    ),
    (
        "Độ sáng và tốc độ cần nằm trong khoảng 1–6.",
        "Brightness and speed must be between 1 and 6.",
    ),
    (
        "Cấu hình qua USB-C hoặc receiver 2.4 GHz.",
        "Configure using USB-C or the 2.4 GHz receiver.",
    ),
    (
        "Chức năng này được cố định trên bàn phím.",
        "This key is fixed by the firmware.",
    ),
    (
        "Kích thước báo cáo HID không được hỗ trợ",
        "Unsupported HID report size",
    ),
    (
        "Kết nối để kiểm tra hỗ trợ đèn hai bên.",
        "Connect to check side-light support.",
    ),
    (
        "Cài đặt đèn hai bên ngoài phạm vi NUT75",
        "Side-light settings are outside the NUT75 range",
    ),
    (
        "Chưa hỗ trợ đổi màu đèn báo trong app.",
        "Indicator color control is not supported in this app yet.",
    ),
    (
        "Kích thước truyền dữ liệu không hợp lệ",
        "Invalid transfer size",
    ),
    (
        "Hồ sơ này không dành cho WEIKAV NUT75.",
        "This profile is not for WEIKAV NUT75.",
    ),
    (
        "Bộ nhớ macro NUT75 vượt quá 512 byte",
        "NUT75 macro memory exceeds 512 bytes",
    ),
    (
        "Cài đặt bàn phím ngoài phạm vi NUT75",
        "Keyboard settings are outside the NUT75 range",
    ),
    (
        "Không mở được Consumer Control NUT75",
        "Could not open NUT75 Consumer Control",
    ),
    (
        "Kiểm tra cáp USB và đóng driver web.",
        "Check the USB cable and close the web driver.",
    ),
    (
        "Phiên bản hồ sơ không được hỗ trợ.",
        "Unsupported profile version.",
    ),
    (
        "Bạn có thể xuất hồ sơ ra tệp JSON.",
        "You can export the profile to a JSON file.",
    ),
    (
        "Thông tin thiết bị cần có 48 byte",
        "Device information must contain 48 bytes",
    ),
    (
        "Địa chỉ macro ngoài bộ nhớ NUT75",
        "The macro address is outside NUT75 memory",
    ),
    (
        "Loại thao tác macro không hỗ trợ",
        "Unsupported macro action type",
    ),
    (
        "Tên hồ sơ cần từ 1 đến 60 ký tự.",
        "Profile names must contain 1 to 60 characters.",
    ),
    (
        "Chức năng phím không được hỗ trợ",
        "Unsupported key binding",
    ),
    (
        "Thiết bị trả về VID khác WEIKAV",
        "The device returned an unexpected WEIKAV vendor ID",
    ),
    (
        "HID long item không được hỗ trợ",
        "HID long items are unsupported",
    ),
    ("Hiệu ứng RGB không được hỗ trợ.", "Unsupported RGB effect."),
    (
        "Không tìm thấy thư mục cấu hình",
        "The configuration directory could not be found",
    ),
    (
        "Thiết bị trả về PID khác NUT75",
        "The device returned an unexpected NUT75 product ID",
    ),
    (
        "Đọc lại đèn hai bên không khớp",
        "Side-light verification failed",
    ),
    ("HID global stack không hợp lệ", "Invalid HID global stack"),
    (
        "Không lưu được thư viện hồ sơ",
        "Could not save the profile library",
    ),
    (
        "Vị trí phím nằm ngoài keymap.",
        "The key position is outside the keymap.",
    ),
    (
        "Chưa thấy giao diện núm NUT75",
        "No NUT75 roller interface was found",
    ),
    (
        "Chọn màu rồi bấm phím để tô.",
        "Choose a color, then click keys to paint.",
    ),
    (
        "Hồ sơ trên máy vẫn được giữ.",
        "Your local profile has been kept.",
    ),
    (
        "Keymap cần có đúng 512 byte.",
        "The keymap must contain exactly 512 bytes.",
    ),
    ("Thư viện hồ sơ không hợp lệ.", "Invalid profile library."),
    ("Hiện gán phím trên bàn phím", "Show key assignments"),
    (
        "Macro có quá nhiều thao tác",
        "The macro has too many actions",
    ),
    (
        "Đọc lại cấu hình không khớp",
        "Settings verification failed",
    ),
    ("Tệp hồ sơ JSON không hợp lệ", "Invalid profile JSON"),
    (
        "Bàn phím không trả lời lệnh",
        "The keyboard did not respond to command",
    ),
    ("Tiêu đề macro không hợp lệ", "Invalid macro header"),
    (
        "HID descriptor bị cắt ngắn",
        "The HID descriptor is truncated",
    ),
    ("SOCD - Hai phím đối hướng", "SOCD - Opposing keys"),
    (
        "Thư viện đã có 100 hồ sơ.",
        "The library already contains 100 profiles.",
    ),
    ("Đổi hiệu ứng đèn cạnh núm", "Next roller-light effect"),
    (
        "Chưa đọc được đèn hai bên",
        "Side-light settings could not be read",
    ),
    ("Ghi độ trễ giữa các phím", "Record delays between keys"),
    ("Chưa hỗ trợ thiết bị này", "Unsupported device"),
    ("Đổi hiệu ứng đèn hai bên", "Next side-light effect"),
    ("Đọc lại macro không khớp", "Macro verification failed"),
    ("Vị trí phím không hợp lệ", "Invalid key position"),
    ("Đổi độ sáng đèn hai bên", "Next side-light brightness"),
    ("Đổi tốc độ đèn cạnh núm", "Next roller-light speed"),
    ("Không khởi tạo được HID", "Could not initialize HID"),
    ("Macro vượt bộ nhớ NUT75", "The macro exceeds NUT75 memory"),
    ("Gói dữ liệu HID quá lớn", "The HID packet is too large"),
    ("Vị trí LED không hợp lệ", "Invalid LED position"),
    (
        "Không đọc được hồ sơ cũ",
        "Could not read the existing profile",
    ),
    ("Đổi tốc độ đèn hai bên", "Next side-light speed"),
    (
        "Số macro phải từ 1–100",
        "Macro numbers must be between 1 and 100",
    ),
    (
        "Dữ liệu ghi thiếu byte",
        "The write payload is missing bytes",
    ),
    ("Đã phát hiện thiết bị", "Device detected"),
    ("Chuột phải để xoá màu", "Right-click to remove color"),
    ("Về danh sách thiết bị", "Back to devices"),
    ("Đổi chức năng con lăn", "Switch roller function"),
    ("Chức năng từ thiết bị", "Device function"),
    ("Không nhập được hồ sơ", "Could not import the profile"),
    ("Dừng", "Stop"),
    ("Đặt lại phím nâng cao", "Reset advanced keys"),
    (
        "Chỉ áp dụng cho lớp đang chọn",
        "Only applies to the selected layer",
    ),
    ("Ẩn", "Hide"),
    ("Hiện", "Show"),
    ("Chạy khi đăng nhập", "Launch at login"),
    (
        "Không thay đổi được cài đặt khởi động",
        "Could not change the login setting",
    ),
    (
        "Cho phép ứng dụng trong cài đặt đăng nhập của hệ thống.",
        "Allow the app in the system login settings.",
    ),
    ("Cầu vồng", "Rainbow"),
    ("Pháo hoa", "Fireworks"),
    ("Trái tim", "Heart"),
    ("Băng hà", "Glacier"),
    ("Mùa xuân", "Spring"),
    ("Ngọc bích", "Jade"),
    ("Trắng", "White"),
    ("Mận", "Plum"),
    ("Cam", "Orange"),
    ("Nhập JSON", "Import JSON"),
    ("Đã gửi hiệu ứng", "Effect sent"),
    ("Chạy khi ứng dụng đang mở", "Runs while the app is open"),
    (
        "Cần kết nối 2.4 GHz hỗ trợ",
        "A supported 2.4 GHz connection is required",
    ),
    ("Không xuất được hồ sơ", "Could not export the profile"),
    (
        "Không kết nối được bàn phím",
        "Could not connect the keyboard",
    ),
    ("Không áp dụng được cấu hình", "Could not apply settings"),
    ("Không tìm được thiết bị", "Could not scan for devices"),
    ("Mất kết nối bàn phím", "Keyboard disconnected"),
    (
        "Không ngắt kết nối được bàn phím",
        "Could not disconnect the keyboard",
    ),
    ("Sắc độ và độ bão hoà", "Hue and saturation"),
    ("Phát / tạm dừng demo", "Play / pause preview"),
    ("Đổi Ctrl / Caps Lock", "Swap Ctrl / Caps Lock"),
    ("Đổi màu đèn cạnh núm", "Next roller-light color"),
    ("Không lưu được hồ sơ", "Could not save the profile"),
    ("Chọn phím kích hoạt", "Choose a trigger key"),
    ("Bộ nhớ macro đã đầy", "Macro memory is full"),
    ("Kết nối lại 2.4 GHz", "Reconnect 2.4 GHz"),
    ("Đổi màu đèn hai bên", "Next side-light color"),
    ("Đèn phím Caps Lock", "Caps Lock key backlight"),
    ("Phím tắt theo hãng", "Vendor shortcut"),
    ("Đang tìm thiết bị…", "Finding devices…"),
    ("Khôi phục mặc định", "Restore default"),
    ("Trễ trước thao tác", "Delay before action"),
    ("Độ trễ", "Delay"),
    ("Chức năng bàn phím", "Keyboard function"),
    ("Chưa kết nối NUT75", "NUT75 is not connected"),
    ("HID report quá lớn", "The HID report is too large"),
    ("Tệp hồ sơ quá lớn.", "The profile file is too large."),
    ("Đèn báo Caps Lock", "Caps Lock indicator"),
    ("Chi tiết thiết bị", "Device details"),
    ("Tự chọn từng phím", "Per-key colors"),
    ("Ưu tiên phím cuối", "Last input priority"),
    ("Tệp hồ sơ quá lớn", "The profile file is too large"),
    ("Tên macro quá dài", "The macro name is too long"),
    ("Vị trí Fn cố định", "Fixed Fn position"),
    ("Lưu màu hiện tại", "Save current color"),
    ("Thiết bị của bạn", "Your devices"),
    ("Chưa có thiết bị", "No devices yet"),
    ("Kết nối bàn phím", "Connect keyboard"),
    ("Kết nối thiết bị", "Connect device"),
    ("Cài đặt bàn phím", "Keyboard settings"),
    ("Nhấn lại để dừng", "Press again to stop"),
    ("CB - Tổ hợp phím", "CB - Key combination"),
    ("Cuộn chuột xuống", "Scroll down"),
    ("Đổi hiệu ứng đèn", "Next lighting effect"),
    ("Hiệu ứng kế tiếp", "Next effect"),
    ("Xuất hồ sơ NUT75", "Export NUT75 profile"),
    ("Nhập hồ sơ NUT75", "Import NUT75 profile"),
    ("Chuột (thiết bị)", "Mouse (device)"),
    ("Media (thiết bị)", "Media (device)"),
    ("HID chỉ ghi được", "HID only wrote"),
    ("+ Thêm thiết bị", "+ Add device"),
    ("Đổi hàng F1–F12", "Swap F1–F12 functions"),
    ("Chong chóng RGB", "RGB windmill"),
    ("Phát / tạm dừng", "Play / pause"),
    ("MT - Chạm / giữ", "MT - Tap / hold"),
    ("TGL - Bật / tắt", "TGL - Toggle"),
    ("Đang tìm NUT75…", "Searching for NUT75…"),
    ("Đã ngắt kết nối", "Disconnected"),
    ("Tăng tốc độ đèn", "Lighting speed up"),
    ("Giảm tốc độ đèn", "Lighting speed down"),
    ("Chưa thấy NUT75", "NUT75 was not found"),
    ("Dùng màu riêng", "Use custom color"),
    ("Chống dội phím", "Debounce level"),
    ("Phím đối hướng", "Opposing key"),
    ("Ưu tiên phím 1", "Key 1 priority"),
    ("Ưu tiên phím 2", "Key 2 priority"),
    ("Đóng thông báo", "Dismiss notification"),
    ("Chuột quay lại", "Mouse back"),
    ("Chuột tiến tới", "Mouse forward"),
    ("Cuộn chuột lên", "Scroll up"),
    ("Hiệu ứng trước", "Previous effect"),
    ("Đang kết nối…", "Connecting…"),
    ("Theo hệ thống", "System"),
    ("Khoá phím Win", "Lock Windows key"),
    ("Thêm thao tác", "Add action"),
    ("Chưa gán phím", "Unassigned"),
    ("Tăng âm lượng", "Volume up"),
    ("Giảm âm lượng", "Volume down"),
    ("Bài tiếp theo", "Next track"),
    ("Bảng chi tiết", "Detailed table"),
    ("Chạy lại demo", "Restart preview"),
    ("Tạm dừng demo", "Pause preview"),
    ("Gán chức năng", "Assign action"),
    ("Quản lý macro", "Edit macros"),
    ("Theo thiết bị", "From device"),
    ("Đã nhập hồ sơ", "Profile imported"),
    ("Đổi hướng đèn", "Reverse lighting direction"),
    ("Đã xuất hồ sơ", "Profile exported"),
    ("Màu bàn phím", "Keyboard finish"),
    ("ĐANG KẾT NỐI", "CONNECTING"),
    ("Chưa kết nối", "Not connected"),
    ("Ngắt kết nối", "Disconnect"),
    ("Hệ điều hành", "Operating system"),
    ("Sao lấp lánh", "Stars"),
    ("Nhấn con lăn", "Press roller"),
    ("Phím / chuột", "Key / mouse"),
    ("Xoá thao tác", "Delete action"),
    ("+  Hồ sơ mới", "+ New profile"),
    ("Đóng kết nối", "Close connection"),
    ("Đang kết nối", "Connecting"),
    ("Tăng độ sáng", "Brightness up"),
    ("Giảm độ sáng", "Brightness down"),
    ("Thiết bị HID", "HID device"),
    ("Đèn hai bên", "Side lights"),
    ("Độ sáng màu", "Color value"),
    ("Ngoại tuyến", "Offline"),
    ("Bắt đầu lại", "Start again"),
    ("Đổi màu đèn", "Next lighting color"),
    ("Tổ hợp phím", "Key combination"),
    ("+  KẾT NỐI", "+  CONNECT"),
    ("Đã kết nối", "Connected"),
    ("Tô toàn bộ", "Fill all"),
    ("Trùng điệp", "Peaks"),
    ("Cuộn xuống", "Rotate left"),
    ("Ghi độ trễ", "Record delays"),
    ("Chuỗi phím", "Key sequence"),
    ("Lặp số lần", "Repeat count"),
    ("Chuột trái", "Left click"),
    ("Chuột phải", "Right click"),
    ("Chuột giữa", "Middle click"),
    ("Đã áp dụng", "Applied"),
    ("Chế độ USB", "USB mode"),
    (" (cố định)", " (fixed)"),
    ("Màu riêng", "Custom Colors"),
    ("Giao diện", "Appearance"),
    ("Không ngủ", "Never"),
    ("Chức năng", "Actions"),
    ("Sáng tĩnh", "Static"),
    ("Chạm sáng", "Reactive"),
    ("Tuyết rơi", "Snow"),
    ("Quang phổ", "Spectrum"),
    ("Bùng sáng", "Flash"),
    ("Dòng chảy", "Stream"),
    ("Tắt tiếng", "Mute"),
    ("Chạm phím", "Key tap"),
    ("Nhấn phím", "Key down"),
    ("Bấm chuột", "Mouse click"),
    ("Macro mới", "New macro"),
    ("Tên hồ sơ", "Profile name"),
    ("Bài trước", "Previous track"),
    ("Dừng phát", "Stop playback"),
    ("Phát demo", "Play preview"),
    ("Trung hoà", "Neutral"),
    ("Đóng bảng", "Close panel"),
    ("Đang chọn", "Selected"),
    ("Hồ sơ mới", "New profile"),
    ("THIẾT BỊ", "DEVICE"),
    ("Chọn màu", "Choose color"),
    ("KIỂM TRA", "KEY TEST"),
    ("Ngôn ngữ", "Language"),
    ("Mặc định", "Default"),
    ("Quét lại", "Scan again"),
    ("Tìm kiếm", "Search"),
    ("Bàn phím", "Keyboard"),
    ("Nâng cao", "Advanced"),
    ("Hoàn tác", "Undo"),
    ("Hiệu ứng", "Effects"),
    ("Chạm tắt", "Reactive off"),
    ("Muôn sắc", "Flowers"),
    ("Nhịp thở", "Breathing"),
    ("Suối màu", "Fountain"),
    ("Sóng màu", "Wave"),
    ("Sóng đôi", "Double wave"),
    ("Gợn sóng", "Ripple"),
    ("Mưa xiên", "Diagonal rain"),
    ("Cuộn lên", "Rotate right"),
    ("Ghi phím", "Record"),
    ("Dừng ghi", "Stop"),
    ("Thả phím", "Key up"),
    ("Thao tác", "Action"),
    ("Nhân bản", "Duplicate"),
    ("Chi tiết", "Details"),
    ("Khi nhấn", "When pressed"),
    ("Đang sạc", "Charging"),
    ("Tuỳ chọn", "Options"),
    ("Bổ trợ 1", "Modifier 1"),
    ("Bổ trợ 2", "Modifier 2"),
    ("Quay lại", "Back"),
    ("Tiến tới", "Forward"),
    ("Đóng ảnh", "Close photo"),
    ("Khoá Win", "Lock Windows key"),
    ("tại byte", "at byte"),
    ("Hai bên", "Side lights"),
    ("ĐÈN NỀN", "BACKLIGHT"),
    ("CÀI ĐẶT", "SETTINGS"),
    ("Cài đặt", "Settings"),
    ("Kết nối", "Connect"),
    ("KẾT NỐI", "CONNECTION"),
    ("Đọc lại", "Refresh"),
    ("Ngủ sau", "Sleep after"),
    ("Áp dụng", "Apply"),
    ("Đèn nền", "Backlight"),
    ("Độ sáng", "Brightness"),
    ("Tắt đèn", "Off"),
    ("Đan sắc", "Cross weave"),
    ("Núi màu", "Mountain"),
    ("Qua lại", "Shuttle"),
    ("Giảm âm", "Volume down"),
    ("Tăng âm", "Volume up"),
    ("+ Macro", "+ Macro"),
    ("Đổi tên", "Rename"),
    ("Một lần", "Once"),
    ("Con lăn", "Roller"),
    ("10 phút", "10 minutes"),
    ("30 phút", "30 minutes"),
    ("Xem pin", "Battery level"),
    ("bản sao", "copy"),
    ("Tốc độ", "Speed"),
    ("Hội tụ", "Converge"),
    ("Sau đó", "Then"),
    ("Bộ nhớ", "Memory"),
    ("1 phút", "1 minute"),
    ("3 phút", "3 minutes"),
    ("5 phút", "5 minutes"),
    ("Hồ sơ", "Profiles"),
    ("Chuột", "Mouse"),
    ("Hướng", "Direction"),
    ("Không", "None"),
    ("PHÍM", "KEYS"),
    ("Sáng", "Light"),
    ("Phím", "Keys"),
    ("Neon", "Neon"),
    ("Trái", "Left"),
    ("Nhấn", "Press"),
    ("Phải", "Right"),
    ("Nhập", "Import"),
    ("Xuất", "Export"),
    ("Chạm", "Tap"),
    ("Phát", "Playback"),
    (" mức", " level"),
    ("phím", "keys"),
    ("Tối", "Dark"),
    ("Màu", "Color"),
    ("Đơn", "Single"),
    ("Thả", "Release"),
    ("Giữ", "Hold"),
    ("Lần", "Times"),
    ("Mở", "Open"),
];

/// Logitech-style compact circular handles; scope changes only the slider, not other controls.
pub(super) fn options_slider(ui: &mut egui::Ui, slider: egui::Slider<'_>) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 20.;
        ui.spacing_mut().button_padding = vec2(6., 2.);
        ui.spacing_mut().slider_rail_height = 3.;
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.));
        let visuals = ui.visuals_mut();
        visuals.handle_shape = egui::style::HandleShape::Circle;
        visuals.slider_trailing_fill = true;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, muted());
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.2_f32, purple());
        visuals.widgets.active.fg_stroke = Stroke::new(1.2_f32, purple());
        visuals.widgets.hovered.bg_fill = surface();
        visuals.widgets.active.bg_fill = surface();
        ui.add(slider)
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_six_new_languages_cover_the_full_ui_catalog() {
        for (_, english) in TRANSLATIONS {
            for language in 2..=7 {
                assert!(
                    super::super::i18n::lookup(english, language).is_some(),
                    "missing locale {language}: {english}"
                );
            }
        }
    }
    #[test]
    fn all_binding_and_lighting_labels_have_english_translations() {
        for label in crate::model::FUNCTIONS
            .iter()
            .map(|(_, name)| *name)
            .chain(crate::model::MEDIA.iter().map(|(_, name)| *name))
            .chain(crate::model::MOUSE.iter().map(|(_, name)| *name))
            .chain(crate::model::LIGHT_MODES.iter().map(|(_, name)| *name))
        {
            let en = translated(label, true);
            assert!(
                en.chars().all(|c| !c.is_alphabetic() || c.is_ascii()),
                "Missing translation: {label}"
            );
            assert_eq!(translated(label, false), label);
        }
    }
    #[test]
    fn composed_errors_translate_without_changing_protocol_values_or_paths() {
        let vi = "Không nhập được hồ sơ: Vị trí phím không hợp lệ: 92";
        assert_eq!(
            translated_message(vi, true),
            "Could not import the profile: Invalid key position: 92"
        );
        let vi =
            "Bàn phím không trả lời lệnh 0x1D tại byte 24. Kiểm tra cáp USB và đóng driver web.";
        assert_eq!(
            translated_message(vi, true),
            "The keyboard did not respond to command 0x1D at byte 24. Check the USB cable and close the web driver."
        );
        assert_eq!(translated_message(vi, false), vi);
        assert_eq!(
            translated_message("Đã xuất hồ sơ: /Users/lit/Hồ sơ/Phím.json", true),
            "Profile exported: /Users/lit/Hồ sơ/Phím.json"
        );
    }
}
