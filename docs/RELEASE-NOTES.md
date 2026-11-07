Weikav Studio 0.9.4 (formerly NUT Studio) is a native desktop companion for the **WEIKAV NUT75** on macOS and Windows.

- Automatically retry connecting when a supported keyboard becomes available.
- Choose Single or RGB with visual previews; the color picker appears for Single and per-key colors.
- Use GIF lighting presets and live lighting presets with a supported 2.4 GHz receiver.
- Optionally launch Weikav Studio when signing in.
- See concise error messages for connection, settings and profile operations.
- Remap keys and the volume roller on Default and Fn layers.
- Test keys, modifier keys, Caps Lock and roller input in the Keys screen.
- Preview 24 lighting modes, use a color wheel and assign per-key colors.
- Record and edit macros with individual keycaps, editable delays, drag-and-drop reordering and single-action deletion.
- Configure SOCD, tap/hold, toggle and key combinations under Advanced Keys.
- Manage device settings, local profiles and keyboard finish.
- Choose Vietnamese, English, Simplified Chinese, Japanese, Korean, French, German or Spanish, with light, dark and system appearance.
- See manufacturer lighting shortcuts in each lighting tab.

**Downloads:** use the Apple Silicon or Intel `.pkg` on macOS; choose the Windows x64 Setup `.exe` or portable `.zip`. `SHA256SUMS.txt` contains checksums.

Configuration uses USB-C or the NUT75 2.4 GHz receiver. Bluetooth configuration is not supported. Side-light and separate indicator controls depend on firmware and are unavailable in the app on the tested 1.40 firmware; manufacturer shortcuts are shown instead. GIF presets and native live lighting presets are experimental and currently available through the 2.4 GHz receiver. GIF editing, AI script import, audio lighting and firmware updates are not included.

Advanced key assignments may fail the post-write verification on the tested keyboard. If this happens, reconnect and check the actual key behavior; the local profile is retained. The Stream effect has not yet been verified against the manufacturer's driver.

Packages are currently unsigned by a platform developer certificate. macOS and Windows builds are produced on their respective operating systems. Physical hardware validation has been performed on macOS; Windows hardware behavior has not yet been verified.

[Website and screenshot gallery](https://luongwnv.github.io/weikav-app/) - [User guide](https://github.com/luongwnv/Weikav-Keyboard-Application#readme). The screenshot archive contains the original screen captures.
