#!/usr/bin/env python3
"""Draw the White NUT75 using the same measured layout as the native preview.
Requires rsvg-convert, Pillow and (for ICNS) macOS iconutil. No vendor photo is modified.
"""
from pathlib import Path
import json, re, subprocess, tempfile
from PIL import Image

root = Path(__file__).resolve().parent.parent
source = (root / 'src/model.rs').read_text()
rows_raw = source.split('const ROWS:')[1].split('let top_spans')[0]
rows = [(list(map(float, re.findall(r'\d+\.', edges))), float(top), float(bottom))
        for edges, top, bottom in re.findall(r'&\[(.*?)\],\s*(\d+\.),\s*(\d+\.)', rows_raw, re.S)]
top_raw = source.split('let top_spans = [')[1].split('];')[0]
top_spans = [(float(a), float(b)) for a, b in re.findall(r'\((\d+\.), (\d+\.)\)', top_raw)]
keys = json.loads((root / 'assets/nut75-layout.json').read_text())['keyList']
svg = ['''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<title>NUT Studio — NUT75 White</title>
<defs><linearGradient id="tile" x2=".6" y2="1"><stop stop-color="#454950"/><stop offset="1" stop-color="#26292e"/></linearGradient>
<linearGradient id="case" x2="0" y2="1"><stop stop-color="#fafbf9"/><stop offset="1" stop-color="#d7dcd8"/></linearGradient>
<linearGradient id="wheel" x2="0" y2="1"><stop stop-color="#525c5d"/><stop offset=".45" stop-color="#d4dddd"/><stop offset=".7" stop-color="#adb9b9"/><stop offset="1" stop-color="#4e595b"/></linearGradient></defs>
<rect x="32" y="32" width="960" height="960" rx="210" fill="url(#tile)"/>
<g transform="translate(512 512) rotate(-18) translate(-500 -200)">
<rect x="68" y="28" width="864" height="384" rx="18" fill="#000" opacity=".22"/>
<rect x="76" y="24" width="848" height="370" rx="12" fill="#a5aca8"/>
<rect x="76" y="15" width="848" height="370" rx="12" fill="url(#case)" stroke="#fff" stroke-width="2"/>''']
for row, items in enumerate(keys):
    edges, top, bottom = rows[row]
    for col, key in enumerate(items):
        if row == 0: left, right = top_spans[col]
        elif row == 5 and col >= 6: left, right = edges[col+1:col+3]
        else: left, right = edges[col:col+2]
        x, y, w, h = left+1, top+1, right-left-2, bottom-top-2
        accent = key['value'] in [0, 61, 83, 76, 88, 89, 90]
        face = '#a3aaa6' if accent else '#f0f1ee'
        svg.append(f'<rect x="{x-1}" y="{y-1}" width="{w+2}" height="{h+2}" rx="4" fill="#5e6763"/>')
        svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="3" fill="#b9bfbb"/>')
        svg.append(f'<rect x="{x+4}" y="{y+3}" width="{w-8}" height="{h-10}" rx="3" fill="{face}"/>')
        # Sparse geometric legends remain legible at Dock sizes; no live font dependency.
        if key['value'] != 83:
            svg.append(f'<rect x="{x+10}" y="{y+10}" width="{6 if w<65 else 12}" height="3" rx="1.5" fill="{ "#edf1ed" if accent else "#78837d"}"/>')
svg.append('''<rect x="866" y="276" width="5" height="15" rx="2" fill="#4ed7a2"/>
<rect x="873" y="271" width="42" height="26" rx="5" fill="#626c68"/>
<rect x="878" y="274" width="32" height="20" rx="3" fill="url(#wheel)"/>
</g></svg>''')
out = root / 'assets/app-icon'
out.mkdir(exist_ok=True)
(out / 'nut75-white.svg').write_text('\n'.join(svg))
subprocess.run(['rsvg-convert', '-w', '1024', '-h', '1024', str(out/'nut75-white.svg'), '-o', str(out/'icon.png')], check=True)
master = Image.open(out/'icon.png')
master.save(out/'icon.ico', sizes=[(n,n) for n in [16,24,32,48,64,128,256]])
with tempfile.TemporaryDirectory() as temp:
    iconset = Path(temp)/'NUT75.iconset'
    iconset.mkdir()
    for n in [16,32,128,256,512]:
        for factor in [1,2]:
            name = f'icon_{n}x{n}' + ('@2x' if factor==2 else '') + '.png'
            master.resize((n*factor,n*factor),Image.Resampling.LANCZOS).save(iconset/name)
    subprocess.run(['iconutil','-c','icns',str(iconset),'-o',str(out/'NUT75.icns')],check=True)
print('Exported NUT75 White SVG, PNG, ICO and ICNS')
