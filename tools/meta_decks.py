"""Playable meta decks: data/meta/archetypes.json -> decks/meta/playable.corpus.json.

An archetype is playable when every card of its list resolves to a pool card
the engine has ported (`diff --list-ported`). The corpus spec holds one deck
per playable archetype (card keys, one entry per copy) and is what
tools/fuzz.py plays; decks/meta/<slug>.txt (tools/fetch_meta.py) are the
readable lists.
"""
import json, os, re, subprocess, collections
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
arch = json.load(open(os.path.join(ROOT, 'data/meta/archetypes.json')))
ported = set(subprocess.run([os.path.join(ROOT, 'engine/target/release/diff'), '--list-ported'], capture_output=True, text=True).stdout.split('\n'))
byk = {(r['set'], r['number']): r for r in pool}
byname = collections.defaultdict(list)
for r in pool:
    byname[r['name']].append(r)
decks = []
for a in arch:
    lines, ok = collections.Counter(), True
    for c in a['cards']:
        r = byk.get((c['set'], str(c['number'])))
        if not r:
            cand = byname.get(c['name'], [])
            r = cand[0] if cand else None
        if not r or r['key'] not in ported:
            ok = False
            break
        lines[r['key']] += int(c['count'])
    if not ok:
        continue
    slug = re.sub(r'[^a-z0-9]+', '-', a['name'].lower()).strip('-')
    decks.append({'name': slug, 'cards': [n for n, q in sorted(lines.items()) for _ in range(q)]})
json.dump({'decks': decks, 'policies': ['bot', 'mix:0.2', 'mix:0.5']}, open(os.path.join(ROOT, 'decks/meta/playable.corpus.json'), 'w'))
print('playable:', [d['name'] for d in decks], [len(d['cards']) for d in decks])
