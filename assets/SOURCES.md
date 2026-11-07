# Asset provenance

Retrieved on 2026-11-07. The UI uses the actual WEIKAV NUT75 image, not a generated approximation or a Logitech keyboard.

## NUT75 image

- File: `nut75.png`, original transparent 1000 × 400 PNG, unmodified.
- Source: https://static.driveall.cn/static/keyboards/NUT75.png
- Referenced by: https://weikav.driveall.cn/custom-keys
- SHA-256: `4ea89527b275335dc06bf2e0a90d4bdaaba10b1f90da40758372eb93482c3162`
- Image/brand rights remain with WEIKAV / DriveAll. No open asset license was published with this image; the code's MIT license does not grant rights to it. Confirm redistribution terms before public distribution.

## Physical layout

- File: `nut75-layout.json`.
- Extracted from the `NUT75` configuration object (`JU`) in the official driver bundle:
  https://weikav.driveall.cn/assets/layout-classic-DGL1ziDE.js
- Original bundle SHA-256 at retrieval: `090b5927ba2d2a49704cb7ec3818d71f5a67133e41597d03721e6625e386496d`
- Preserves the manufacturer's 81 physical key positions, HID usages, matrix indices and three roller action indices.
- The six row hitboxes in `src/model.rs` align with the original image coordinates. They do not infer positions from a generic 75% keyboard.

## Inter font

- File: `InterVariable.ttf`.
- Source: https://github.com/rsms/inter/blob/master/docs/font-files/InterVariable.ttf
- License: SIL Open Font License 1.1; full text in `Inter-LICENSE.txt`.

The supplied Logitech screenshots were used only as visual references for spacing, navigation and the action inspector. Their images, marks and watermarks are not included in the app.

## NUT75 colorway photographs (0.2)

The following original, unmodified vendor photographs are embedded for the **Cài đặt → Ảnh** gallery:

- `colorways/black.jpg`: https://weikav.com/wp-content/uploads/2026/06/BLK.jpg
- `colorways/silver.jpg`: https://weikav.com/wp-content/uploads/2026/06/WHE.jpg
- `colorways/ice-blue.jpg`: https://weikav.com/wp-content/uploads/2026/06/IBE.jpg
- `colorways/orange.jpg`: https://weikav.com/wp-content/uploads/2026/06/ORG.jpg
- `colorways/white.jpg`: https://weikav.com/wp-content/uploads/2026/06/SIR.jpg
- `colorways/frost-silver.png`: https://weikav.com/wp-content/uploads/2026/07/frost-silver-sku.png

Color names/SKU associations were verified against https://weikav.com/product/weikav-nut75/ on 2026-11-07. These photographs share the same ownership/redistribution caveat as the driver image above.

Colorways other than the original Ice Blue top view are drawn as native egui illustrations in the interactive editor. They are identified as illustrations and are not passed off as vendor top-view renders. The lighting preview is a code-rendered visual approximation; no AI-generated keyboard imagery is used. The roller samples the original driver photograph with texture UVs to preserve its actual appearance.

## Application icon (0.7)

`app-icon/nut75-white.svg` is a native vector illustration of the White NUT75 using the same measured 81-key geometry as the preview; no photograph was recolored or passed off as a manufacturer render. `scripts/generate-app-icon.py` exports PNG, ICNS and multi-resolution ICO. The Windows build embeds the ICO using the Windows SDK resource compiler; the native window also uses the PNG.

## Advanced Keys icons (0.8)

`advanced/{socd,mt,tgl,cb}.svg` are the exact vendor SVG masks extracted from `i-svg:socd`, `i-svg:mt`, `i-svg:tgl` and `i-svg:cb` in https://weikav.driveall.cn/assets/main-oBrmcLli.css on 2026-11-07. Corresponding PNG masks are rendered at 96 px and tinted with the app text/accent color at runtime. Ownership remains with the vendor as for the keyboard artwork.


## CJK font (0.9)

`fonts/NotoSansCJKsc-Regular.otf` is the original Noto Sans CJK SC Regular font from https://github.com/notofonts/noto-cjk/raw/refs/heads/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf. It provides the fallback glyphs for Simplified Chinese, Japanese and Korean. Licensed under SIL OFL 1.1; bundled in `fonts/NotoSansCJK-LICENSE.txt` and desktop packages.

## Lighting shortcut guide (0.9)

Key combinations were transcribed from the NUT75 Quick Start Guide supplied by the user: https://drive.google.com/file/d/12tl04Z6Ol5kfUWCdcbDVPiEMCkWKF_vb/view. The application links to this document and renders keycaps using its existing model. No manual artwork is copied into the package. Fn+Delete/Home are described as the indicator next to the knob, separately from the Caps Lock key’s backlight.
