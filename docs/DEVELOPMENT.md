# Development and packaging

Weikav Studio uses Rust and egui/eframe. There is no Electron, browser wrapper or embedded web driver. Protocol notes and hardware observations are in this directory.

## Requirements

Rust stable (minimum 1.88), Xcode Command Line Tools on macOS, or MSVC Build Tools and the Windows SDK on Windows. Inno Setup 6 is needed to make the Windows installer.

```sh
cargo run --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

## Packages

```sh
scripts/package-macos-pkg.sh
```

On Windows:

```powershell
scripts/package-windows.ps1 -Installer
```

Release packages go to `dist/release`, which is ignored by Git. macOS packages install to `/Applications/Weikav Studio.app`. The packages are currently unsigned by a platform developer certificate. Only app bundles are ad-hoc signed locally.

Push a `v<version>` tag matching `Cargo.toml` to build Apple Silicon/Intel macOS packages, a Windows x64 installer and a portable archive. The release workflow tests each platform, generates SHA-256 checksums and publishes a release only after all build jobs pass. A normal main-branch push runs the same build without publishing a release.

The source repository is `luongwnv/Weikav-Keyboard-Application`. The static English site in `website/` is published separately to `luongwnv/weikav-app` for the requested GitHub Pages path.

## Assets and localization

Provenance and licenses are in `assets/SOURCES.md`. Keep all bundled font licenses in packages. `assets/locales/catalog.json` contains six additional locale columns (Simplified Chinese, Japanese, Korean, French, German, Spanish), keyed by the English UI string. Coverage, composed-error and CJK glyph tests guard translations and paths.

Never use hardware writes as a UI screenshot or test fixture. A local profile is a set of selective changes, not a full device-memory backup.
