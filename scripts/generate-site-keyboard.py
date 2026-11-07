#!/usr/bin/env python3
"""Export the native White NUT75 model for the website, using the app's measured layout."""
from pathlib import Path
import base64
import html
import json
import re

root = Path(__file__).resolve().parent.parent
source = (root / 'src/model.rs').read_text()
rows_raw = source.split('const ROWS:')[1].split('let top_spans')[0]
rows = [(list(map(float, re.findall(r'\d+\.', edges))), float(top), float(bottom))
        for edges, top, bottom in re.findall(r'&\[(.*?)\],\s*(\d+\.),\s*(\d+\.)', rows_raw, re.S)]
top_raw = source.split('let top_spans = [')[1].split('];')[0]
spans = [(float(a), float(b)) for a, b in re.findall(r'\((\d+\.), (\d+\.)\)', top_raw)]
keys = json.loads((root / 'assets/nut75-layout.json').read_text())['keyList']
svg = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="64 3 872 400" role="img" aria-labelledby="title"><title id="title">WEIKAV NUT75 White — native app model</title>',
       '<rect x="76" y="19" width="848" height="370" rx="10" fill="#929392"/>',
       '<rect x="76" y="15" width="848" height="370" rx="9" fill="#edeeed" stroke="#b1b2b1" stroke-width="1.2"/>',
       '<rect x="78" y="17" width="844" height="366" rx="7" fill="none" stroke="#fff"/>']
for row, items in enumerate(keys):
    edges, top, bottom = rows[row]
    for col, key in enumerate(items):
        if row == 0: left, right = spans[col]
        elif row == 5 and col >= 6: left, right = edges[col+1:col+3]
        else: left, right = edges[col:col+2]
        x,y,w,h = left+1,top+1,right-left-2,bottom-top-2
        kid=key['value'];accent=kid in [0,76,83,88,89,90,91]
        face='#a3aaa6' if accent else '#eff1ee'
        base='#7b817e' if accent else '#b5b7b4'
        label=key['name']
        mapping={16:'~',27:'−',28:'=',43:'[',44:']',58:';',59:"'",60:'\\',72:',',73:'.',74:'/',64:'Shift',75:'Shift',80:'Ctrl',87:'Ctrl',81:'Win',82:'Alt',83:''}
        if kid in mapping: label=mapping[kid]
        elif 17<=kid<=26: label=label.split()[0]
        size=9.5 if len(label)>4 else 15 if len(label)==1 else 10.5
        svg += [f'<rect x="{x-1.1}" y="{y-1.1}" width="{w+2.2}" height="{h+2.2}" rx="3" fill="#3b3b3b"/>',
                f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="3" fill="{base}"/>',
                f'<rect x="{x+4}" y="{y+3}" width="{w-8}" height="{h-10}" rx="2.5" fill="{face}"/>',
                f'<path d="M{x+6} {y+3}H{x+w-6}" stroke="#fff" stroke-width="1" opacity=".8"/>',
                f'<text x="{x+w/2}" y="{y+3+(h-10)/2}" dominant-baseline="central" text-anchor="middle" font-family="Inter,Arial,sans-serif" font-size="{size}" fill="{ "#eff2f0" if accent else "#53625b"}">{html.escape(label)}</text>']
photo=base64.b64encode((root/'assets/nut75.png').read_bytes()).decode()
svg += ['<rect x="864.8" y="274.8" width="7.4" height="17.4" rx="2" fill="#414141"/>',
        '<rect x="866" y="276" width="5" height="15" rx="1.5" fill="#787878"/>',
        '<rect x="873" y="271" width="42" height="26" rx="3" fill="#9a9b9a"/>',
        '<rect x="875" y="273" width="38" height="22" rx="3" fill="#3f3f3f"/>',
        f'<svg x="879" y="275" width="32" height="18" viewBox="879 275 32 18"><image href="data:image/png;base64,{photo}" width="1000" height="400"/></svg>', '</svg>']
(root/'website/assets/nut75-white.svg').write_text('\n'.join(svg))
print('Exported the native model: 81 measured keys and original roller texture')
