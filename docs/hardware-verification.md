# Physical NUT75 verification

Date: 2026-11-07. Host: macOS, Apple Silicon. Connection: 2.4 GHz.

After the user confirmed that the keyboard was awake, the release binary completed its read-only diagnostic successfully:

```sh
./target/release/weikav-studio --list-devices
NUT75_TRACE_HID=1 ./target/release/weikav-studio --probe
```

Observed results:

| Field | Observed value |
|---|---|
| Product string | NUT75 2.4G |
| VID / PID | 0C45 / FEF9 |
| HID usage page / usage | FF60 / 61 |
| Interface | 3 |
| Unnumbered report size | 64 bytes |
| Firmware | 1.40 |
| Battery at verification | 56% |
| Firmware frame version | 0 |
| Manufacturer / product codes | 17 / 28 |
| Firmware work mode | 2 |
| Hardware profile index | 0 (displayed as profile 1) |
| Base keymap read | 512 bytes, all 10 chunks acknowledged |
| Fn keymap read | 512 bytes, all 10 chunks acknowledged |
| RGB read | 16 bytes |
| RGB mode | 1 (static) |
| RGB color | FF0328 |
| RGB brightness / speed | 2 / 3 |
| RGB color mode | Single color |

All read responses had the expected `55` header, matching command and sufficient data. Addresses in observed keymap responses also matched the requested addresses. No retries were needed in this successful diagnostic.

The native GUI was subsequently connected to the same receiver. It closed the connection dialog, showed **Đã kết nối**, and displayed firmware 1.40, battery 56%, HID 64 bytes, connection 2.4 GHz and the hardware identity above.

Earlier timeouts were resolved by physically waking the keyboard. The current app reports a read failure and requires reconnecting if it cannot communicate; it does not keep a sleeping device marked as actively connected.

This initial verification sent only read commands. Apply was not pressed during this initial check. See the subsequent observations below for the user-initiated RGB write. Unit tests and Windows target compilation do not substitute for physical checks.

## 0.4 read verification

The updated native GUI connected successfully to the same receiver and read custom LED (512 bytes), game-mode/settings (56 bytes) and the macro index in addition to the original keymaps/RGB. It showed firmware 1.40 and battery 49% at this check. Applying changes was not performed. New macro, advanced binding, custom LED and settings writes remain unverified on physical hardware.

## 0.6 observations

- Read-only consumer HID diagnostic identified report ID 3, a 16-bit array. The user rotated both directions and pressed the wheel, and explicitly confirmed all three visual indicators lit. The same monitor is now integrated into Keys; no separate Test menu remains. Tested keys keep a green overlay; wheel directions/press are marked on the wheel instead of three separate chips below the keyboard. The monitor refreshes its interfaces when USB/receiver devices change.
- The user initiated an RGB Apply. A subsequent read-only probe returned mode 13, RGB 84/50/FF, brightness 4, speed 3, direction 0. The local draft retained direction 2 from a previous directional mode. The old exact comparison incorrectly reported a mismatch for this unused field. Comparisons now ignore unused mode-specific fields; tests cover real mismatches too. A new physical write has not been performed to validate the fix.
- `--probe-zones` read generic side (`1D`) and light-box (`1B`) commands twice; both returned 24 zero bytes. This does not establish configurable support. Native side writes are gated behind a valid nonzero state and readback. No side/light-box write was performed.
- The user reported that Fn + Backspace did not change the side lights. A fresh keymap read shows Fn matrix slot 92 still maps to FUNC 23 (side-effect cycle), and base Fn slot 85 remains keyboard usage AF. Firmware 1.40, frame 0. This remains unresolved; USB-C comparison is needed.
- The official NUT75 manual linked by https://weikav.com/waikv-software/ documents Fn+Backspace for side effects, Fn+Delete for the wheel indicator color and Fn+Home for its mode. It labels a separate Caps Lock indicator but documents no configurable indicator color. The app distinguishes this indicator from the editable backlight of physical key 48.

USB-C, Windows hardware, custom-per-key writes, side writes, macro writes and advanced binding writes remain unverified.

### Subsequent USB-C comparison

After the user connected USB-C and selected Fn+T, enumeration returned NUT75, VID/PID 0C45/880C, usage page FF68/usage 61, interface 3. A read-only `--probe-zones` succeeded over USB: firmware 1.40/frame 0, full keymaps/settings/custom LED readable, main mode 11. Both generic side and light-box records again contained 24 zero bytes. This remains insufficient to enable those writes. USB configuration reads are now verified; USB writes and auxiliary indicator controls remain unverified. USB battery data returned 0 and is omitted from the UI instead of presented as an empty battery.

The updated home screen was observed automatically connecting to the USB device and displaying Connected / USB-C. Settings showed Connected, USB-C, firmware 1.40. The user had previously connected the 2.4G receiver, which remained in history as Offline after removal.

### Focused macOS media-event fix

The user reported no wheel visual after integrating the tester into Keys. A bounded Consumer Control diagnostic received actual USB reports `03 E9 00` and `03 EA 00`, correctly decoded, while the GUI's direct HID open was denied with IOKit `0xE00002E2` (not permitted). Terminal permission was not evidence that the signed app could open input HID.

The macOS GUI now uses an AppKit **local** `NSEvent` monitor for SystemDefined media key presses. It observes only events dispatched to this app, checks focused Keys through its active flag, returns every event unchanged and installs no global tap. No Input Monitoring or Accessibility grant was performed. Direct HID monitoring remains for Windows and as a fallback if installing a local monitor fails. Local media events can originate from any keyboard, like ordinary focused key test events; they are not proof of NUT75 device identity.

During the physical retest, local media events reached the GUI and the wheel gained a persistent green outline and direction marks. The user explicitly answered “Đã hiện”. Mute decoding is covered with the same SDK key format. Native media event constants were checked against the installed macOS SDK `IOLLEvent.h` and `ev_keymap.h`. Temporary diagnostic logging was removed from the source after identifying the cause.

Visual checks also verified equal-height Settings cards, light/dark switching, and a key remaining green after release. Thumbnail labels now scale with the full keyboard; the reset button retains the previous glyph and aligns by its painted bounds.

### Caps Lock visual and custom-mode alias

The user clarified the missing Caps Lock light was the indicator in the app, not a physical color-control request. The NUT75 manual places this status LED immediately left of the wheel. The preview now draws the separate LED there using the host Caps Lock state; it is independent of main RGB/custom key lighting. macOS local FlagsChanged events wake the UI, and Windows reads the lock toggle through GetKeyState. The user physically toggled Caps Lock and explicitly confirmed the on/off visual was correct.

A subsequent user-initiated custom/per-key Apply produced raw RGB mode **20** while the UI draft requested **128**. The vendor's current lighting controller explicitly treats modes `[20,21,128]` as custom mode. Native reads now normalize 20/21 to 128, avoiding a false mismatch; writes retain the vendor UI's 128 value. Direction defaults are also normalized (cross weave uses 3, other directional modes 0) and the picker exposes only the supported axis. Added tests cover the alias and direction behavior; a new hardware write after this fix has not been performed by the agent.

Bluetooth HID/GATT discovery was also completed read-only; see bluetooth-investigation.md. The custom FEE1 characteristic described itself as OTA Channel and was never written.

### Final UI review

Native UI observation verified the thumbnail list with proportional labels (no overlapping legends), equal-height Settings cards, Macro trigger/action flow with a content gutter, and the capsule lighting-zone selector. Color/brightness/speed/undo now sit below the keyboard while all effect tiles remain on the right. A real read-only Settings Refresh produced a success toast whose close circle visibly overlaps the top-right border, as requested. The same close component is used by error toasts. The host Caps Lock and macOS media visuals were physically confirmed by the user.

## Modifier input (0.7.1)

The macOS local NSEvent monitor handles FlagsChanged, using Carbon virtual key codes and the device-specific masks from the installed SDK IOLLEvent.h to distinguish left/right Shift, Control, Option and Command. It forwards the event unchanged and is enabled only in the focused Keys view. Fn is mapped only when macOS actually reports virtual key 63; the firmware-local Fn key is not inferred from unrelated key presses. Windows reads side-specific GetKeyState values in the focused view.

References: [Apple FlagsChanged](https://developer.apple.com/documentation/appkit/nsevent/eventtypemask/flagschanged), [Microsoft GetKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeystate). Unit coverage includes both Shift keys held together and one released. On 2026-11-07 the user confirmed that both Shift keys, both Control keys, Win and Alt retain green test marks in version 0.7.1. Fn remains conditional on firmware reporting it to the OS.

## Lighting diagnostics, 0.8.0 (2026-11-07)

The user reported that Cross weave did not follow the chosen red color on the physical NUT75 receiver/firmware 1.40, although GET_LED_EFFECT returned mode 10, RGB (255,79,84), colorMode 0. A reversible lighting-only test set static red mode 1, brightness 5; the user confirmed red, stationary LEDs. The original 16-byte lighting record was restored from a temporary backup. The user confirmed that restored Cross weave still ran other colors. No keymaps, macros or keyboard settings were written during this diagnostic. The app disables monochrome selection and uses an RGB preview for this observed firmware/mode combination; it does not claim this limitation for firmware versions that have not been tested.

The vendor read query 0x3B (GET_ALL_LIGHTS_RGB_24G_64_BYTE) returned zero for all 128 queried RGB565 values even while the user saw lit LEDs; this endpoint is not used as a reliable live preview on this firmware. CLI --probe-led-colors is read-only. CLI --test-red-light explicitly writes static red and prints its temporary backup path; --restore-light <path> restores only that lighting record.

RGB readback now ignores brush RGB when colorMode is RGB and retries reads for up to 500 ms after a write, without repeating writes. Matching still checks mode, brightness, applicable speed and direction; single-color modes still verify RGB.


## UI 0.8.2

- Người dùng xác nhận kéo macro đã mượt và các dòng trượt nhường chỗ. Sửa khóa lồng nhau của egui Context khi lưu điểm nắm chuột; bổ sung kiểm tra chuỗi nhấn, di chuyển và thả chuột chạy qua renderer danh sách thực. Không ghi phần cứng để kiểm tra giao diện.
- Đã xem trực tiếp dropdown có dấu chọn, bố cục Device hai phần, ô màu tròn, slider cùng đường tâm và nút đóng chồng lên góc viền hộp thoại. Người dùng xác nhận nút đã nằm ra ngoài.
- Thông báo thành công ở trên bên phải; lỗi ở trên bàn phím bên trái trong Keys/Backlight/Advanced.


## UI 0.8.3

Theo yêu cầu cập nhật, nút đóng đã chuyển hoàn toàn vào trong góc phải trên của hộp thoại, bảng màu và thông báo; thông báo chừa khoảng riêng để không đè chữ. Đã kiểm tra trực tiếp hộp thoại kết nối. Phím tắt Fn + Backspace dùng chung renderer keycap với Macro, lấy từ mô hình và màu vỏ hiện tại; đã kiểm tra trực tiếp ở Đèn nền - Hai bên.


## UI 0.8.4

Theo yêu cầu phân biệt mới, nút X của thông báo chồng lên viền góc phải trên, còn nút đóng hộp thoại và bảng màu nằm trong. Đã xem trực tiếp thông báo Đã kết nối lúc khởi động. Danh sách kết nối thay icon bàn phím chung bằng thumbnail cùng mô hình và màu vỏ NUT75 đang dùng ở trang thiết bị; không gán hình NUT75 cho thiết bị không được nhận diện. Đã kiểm tra trực tiếp.


## UI 0.8.5

Mỗi mục ở danh sách thiết bị có nút X hiện khi hover hoặc điều hướng bàn phím tới mục. Xoá mục đang kết nối sẽ đóng phiên HID cấu hình, không đổi bộ nhớ bàn phím hay hồ sơ. Thiết bị đã xoá được lưu trong hidden_devices để quét và khởi động không tự thêm lại; kết nối thủ công thành công sẽ hiện lại. Kiểm thử xác nhận lưu/đọc, xoá lặp không trùng, giữ thiết bị khác và hồ sơ, thêm lại thành công. 35 kiểm thử đạt, Clippy đạt, kiểm tra target Windows đạt. Không xoá thiết bị thực để kiểm thử.
