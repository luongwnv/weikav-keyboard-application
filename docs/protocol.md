# NUT75 HID implementation

This is an independent client of the protocol used by the official WEIKAV web driver, not VIA/QMK. The transport and serialization were reimplemented in Rust from publicly served driver behavior. No vendor JavaScript is executed by the app.

Sources inspected on 2026-11-07:

- https://weikav.driveall.cn/custom-keys
- https://weikav.driveall.cn/assets/layout-classic-DGL1ziDE.js
- https://weikav.driveall.cn/assets/lighting-DzPku7KQ.js
- https://docs.rs/hidapi/2.6.7/hidapi/struct.HidDevice.html

## Identification

| Connection | VID | PID | Additional requirement |
|---|---|---|---|
| USB | `0x0C45` (3141) | `0x880C` (34828) | Product name contains NUT75 |
| 2.4 GHz | `0x0C45` (3141) | `0xFEF9` (65273) | Product name contains NUT75 |

Configuration usage pages are filtered against the driver's own set: `FF68`, `FF80`, `FF60`, `FF00`, `FF01`, `FF1B`. The NUT75 2.4G interface observed locally uses usage page `FF60`, usage `61`, and 64-byte unnumbered reports. Generic receivers are listed but cannot be opened for configuration. NUT87 can share a PID; VID/PID alone are insufficient.

The app additionally validates the device-info VID/PID and refuses recovery-mode firmware. USB identity is checked before writes; the same open session is used throughout. macOS uses nonexclusive HID access (`macos-shared-device`) so the app does not seize the keyboard. Windows uses hidapi's native Windows backend.

## Packets

Output report length is parsed from the HID report descriptor. Supported payload lengths are 32 or 64 bytes, report ID 0. No fallback guess is used. `hidapi::write` receives an extra leading report-ID byte.

| Offset | Request | Response |
|---|---|---|
| 0 | `AA` | `55` |
| 1 | Command | Command |
| 2 | Chunk length | Command-dependent |
| 3–4 | Little-endian byte address | Address |
| 5 | 0 | Reserved |
| 6 | Last-chunk flag | Reserved |
| 7 | 0 | Reserved |
| 8… | Data for write commands | Data for read commands |

The data capacity per packet is `report_length - 8`. Each exchange is serialized in a dedicated worker thread. Stale replies are drained before a request. Responses must have the expected header, command, and sufficient length. Like the web driver, the implementation does not require an echoed response address. Each chunk has at most three attempts. Normal timeout is 750 ms; frame version 1 uses 2 seconds.

## Supported operations

| Command | Meaning | Size |
|---|---|---|
| `10` | Read device info | 48 bytes |
| `12` | Read base keymap | 512 bytes |
| `16` | Read Fn keymap | 512 bytes |
| `13` | Read RGB effect | 16 bytes |
| `22` | Write base keymap | 512 bytes |
| `26` | Write Fn keymap | 512 bytes |
| `23` | Write RGB effect | 16 bytes |

Version 0.4 also implements GET/SET_GAME_MODE (`11`/`21`, 56 bytes), GET/SET_CUSTOM_LED_DATA (`14`/`24`, 512 bytes), and GET/SET_MACRO (`15`/`25`, 400-byte index plus blocks). Firmware, bootloader, factory reset and streaming animation/audio commands are not implemented. See feature-parity.md for remaining web-driver functions.

Each keymap has 128 four-byte records indexed by the manufacturer's matrix ID. Supported editable records:

- Default: `[0, 0, 0, 0]` (firmware decides the factory function).
- Keyboard: `[2, 0, HID_usage, 0]`.
- Consumer/media: `[3, usage_low, usage_high, 0]`.
- Mouse button: `[1, 1, bit_mask, 0]`, masks `1, 2, 4, 8, 16`.
- Mouse wheel: `[1, 3, direction, 0]`, direction `1` or `255`.

Unknown/advanced records remain byte-for-byte unchanged unless that exact physical slot is edited. Fn is kept accessible, F1–F12 are locked on the Fn layer as in the official configuration, and roller actions are edited on the base layer.

RGB modes use the official IDs: 0 off, 1–19 basic effects, 23–25 NUT75 extras and 128 custom LED. All 24 choices have native icons and approximate animated previews. Brightness/speed are 1–6 for NUT75. Direction is editable for supported effects (10, 11, 12, 16, 18, 23, 25). Secondary RGB, effect metadata and unused bytes are preserved; the write marker is `AA 55` at offsets 14–15.

## Additional formats in 0.4

- Macro binding: `[6, index, mode, count]`; modes `0` routine, `1` repeat count, `2` press again to stop. Names are local. The 100-entry index stores little-endian u32 addresses; nonempty blocks begin at 400. A block has u16 `2 * action_count`, two reserved zeros, then records `[delay_lo, delay_hi, key, status]`. Keyboard press/release: `90/10`; mouse: `B0/30`. NUT75 allows 512 bytes of blocks, excluding index. Index packets omit the final flag while a body follows; only its last packet commits. Reads validate every address/count before requesting data. Unchanged device macros are merged before repacking, not discarded.
- CB: `[7, modifier1, modifier2, key]`, modifier usages E0–E7 or zero.
- MT: `[9, tap, hold, delay_ms / 10]`.
- TGL: `[10, key, 0, 0]`.
- SOCD: `[11, mode, key1, key2]` assigned to both physical keys; 1/2 absolute key1/key2 priority, 3 last input priority, 4 neutral.
- FUNC: `[13, 0, 0, function_id]` for exposed standard NUT75 functions.
- Custom LED: 128 records `[led_id, r, g, b]`. Selected color patches preserve all other records.
- Game mode: fields at 1 gameMode, 2 fnSwitch, 3 sleep in minutes, 4 debounce **level 1–5** (not milliseconds), 5 reportRate (3=1000, 5=4000, 6=8000), 6 systemMode, 17 NKRO. Other bytes are preserved. Wireless-specific report rate at 18–19 is preserved rather than guessed.
- The official translation endpoint [English labels](https://config.driveall.cn/langs/en.json) confirms macro/SOCD labels and debounce level units.

## Apply semantics

Local profiles contain only explicit patches, not a device backup. Switching profiles does not automatically write to the device. An empty profile preserves the current device state; it is not a factory reset.

On Apply, the app validates the profile, freshly reads both layers, RGB, custom LED, game mode and macro data, merges only selected edits, writes only changed sections, and reads each written section back. Success is shown only after byte-for-byte keymap verification and comparison of edited RGB fields. If a later section fails, earlier sections may already have been written. The connection is dropped, the local profile is retained, and the user is asked to reconnect and read the actual device state. There is no claim of an atomic hardware transaction.

## Verification status

Unit tests cover packet headers, 32/64-byte chunking, descriptor parsing, key indices, identity checks, protected positions, import validation, mouse encoding and preservation of unedited bytes.

The macOS GUI was run and checked with native UI automation. A connected `NUT75 2.4G` receiver was enumerated and its descriptor read successfully. Initial reads timed out while the keyboard was asleep. After the user woke it, `GET_DEVICE_INFO`, `GET_KEY`, `GET_FN_KEY` and `GET_LED_EFFECT` all succeeded over 2.4 GHz, and the GUI connected and displayed the actual firmware/battery information. Firmware was 1.40 and battery was 56% at the time of verification. See `hardware-verification.md` for the observed configuration.

See hardware-verification.md for the subsequent user-initiated RGB Apply and its false mismatch caused by an unused direction field. USB-C reads subsequently succeeded; other write operations and Windows hardware remain unverified. No firmware update was performed.

## 0.6 auxiliary lighting and input

The generic vendor protocol has GET/SET_SIDE_LIGHT (`1D`/`2D`), 24 bytes. Fields are mode 0, RGB 1–3, color mode 8, brightness 9, speed 10. Native merging preserves all other bytes and requires readback equality for the modeled fields. Side modes 1 flow, 2 static, 3 breathing, 4 spectrum, 5 off, 6 scrolling colors. The observed USB and receiver firmware returns all zeros; this is treated as unknown/unsupported, not off. Side writes are rejected before any profile section is written unless a valid state has been read. A profile can carry an optional `side_lighting` patch; older profiles default to none.

Caps Lock key backlight is the existing custom LED record at physical matrix ID 48. Editing it requires mode 128, which changes the main keyboard to per-key lighting. This does not control the separate Caps Lock status indicator. No verified status-indicator endpoint is known.

Read-only Consumer Control HID monitoring uses the device's report descriptor to parse arrays and variables, including report IDs and bit offsets. E9/EA/E2 trigger volume up/down/mute visual pulses. It runs only on focused Keys, without seizing the device or installing a global keyboard hook. The user confirmed the three pulses before the tester was merged into Keys.

RGB readback comparison ignores direction for non-directional effects, speed for static/custom, and brush color for custom or inherently multicolor effects. It still checks meaningful fields; an unused firmware field no longer produces a false failure.

On macOS the app prefers AppKit local SystemDefined media events to avoid requiring direct input-HID permission. The monitor is created/removed on the UI thread, accepts press/repeat states for SDK media key types 0/1/7, and returns the original event unchanged. Its active flag follows focused Keys. It neither installs a global monitor nor attributes the event to a specific keyboard. See https://developer.apple.com/documentation/appkit/nsevent/addlocalmonitorforevents(matching:handler:). The raw HID parser remains used for Windows and diagnostics.

NUT75 firmware 1.40 reports custom mode 20 after a mode-128 write. The vendor lighting controller explicitly recognizes 20, 21 and 128 as custom lighting; native decoding normalizes these to UI mode 128. Cross-weave direction defaults to 3 (vertical), other directional effects to 0 (horizontal). Unsupported direction values are normalized before writing/comparing. Caps Lock preview uses the host lock toggle and never writes lock LEDs or firmware settings.

Bluetooth report and GATT findings are documented in bluetooth-investigation.md; the OTA characteristic is not used for configuration.
