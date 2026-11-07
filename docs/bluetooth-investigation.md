# NUT75 Bluetooth investigation — 2026-11-07

The user switched the paired NUT75 to Bluetooth and confirmed typing worked. All investigation below was read-only: HID enumeration/descriptors, CoreBluetooth service/characteristic/descriptor discovery, and GATT value reads. No GATT characteristic write or firmware operation was sent.

## Observed device

The keyboard was exposed as `LIT NUT75`, Bluetooth bus, HID VID/PID `0000/0000`. Its HID descriptor contains keyboard (including NKRO and modifiers), system control, mouse and Consumer Control collections. Consumer reports use report ID 8 on Bluetooth, whereas USB uses ID 3.

The only HID Output report is report ID 2 with one byte for standard keyboard lock LEDs (three LED bits plus padding). No Feature report or 32/64-byte vendor configuration Output report was present. This is insufficient to carry the USB/2.4G DriveAll protocol.

## GATT discovery

| Service | Characteristic | Observed properties | Read result |
|---|---|---|---|
| `180F` Battery | `2A19` | Read + Notify (`18`) | `5E`, standard battery value 94% at this check |
| `FEE0` | `FEE1` | Read + Write Without Response + Write (`14`) | Empty value |

The `2901` Characteristic User Description of `FEE1` was **OTA Channel**. It must not be treated as a configuration channel merely because it has write properties. No writes were attempted. This is a concrete hardware observation, not an assumption from UUID names.

## Feasibility and limits

A Rust Bluetooth client can identify the keyboard, read its standard battery service, and use ordinary focused input testing. The app's local macOS input monitor already receives media/Caps Lock events independently of the USB configuration session.

No keymap/RGB configuration endpoint was identified in the HID descriptor or the GATT services returned to this client. Therefore this version does not claim Bluetooth configuration support. Implementing it requires a documented vendor protocol or firmware that exposes a suitable configuration service. This investigation does not prove that every firmware revision has the same services.

The native read-only HID diagnostic is `cargo run --locked -- --probe-bluetooth`. The temporary CoreBluetooth inspection was a development probe, not part of the shipped Rust app and not a production Bluetooth backend.
