"""Convert data/meta/archetypes.json lists to Twinleaf full names.

Writes decks/meta-tl/<slug>.txt for every archetype whose cards all map to
ported cards, plus decks/meta-tl/playable.corpus.json (corpus spec)."""
import json, os, re, subprocess, collections
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
arch = json.load(open(os.path.join(ROOT, 'data/meta/archetypes.json')))
ported = set(subprocess.run([os.path.join(ROOT, 'engine/target/release/diff'), '--list-ported'], capture_output=True, text=True).stdout.split('\n'))
byk = {(r['set'], r['number']): r for r in pool}
byname = collections.defaultdict(list)
for r in pool:
    if r.get('fullName'):
        byname[r['name']].append(r)
os.makedirs(os.path.join(ROOT, 'decks/meta-tl'), exist_ok=True)
decks = []
for a in arch:
    lines, ok = collections.Counter(), True
    for c in a['cards']:
        r = byk.get((c['set'], str(c['number'])))
        if not r or not r.get('fullName'):
            cand = byname.get(c['name'], [])
            r = cand[0] if cand else None
        if not r or r['fullName'] not in ported:
            ok = False
            break
        lines[r['fullName']] += int(c['count'])
    if not ok:
        continue
    slug = re.sub(r'[^a-z0-9]+', '-', a['name'].lower()).strip('-')
    open(os.path.join(ROOT, 'decks/meta-tl', slug + '.txt'), 'w').write(''.join('%d %s\n' % (q, n) for n, q in sorted(lines.items())))
    decks.append({'name': slug, 'cards': [n for n, q in sorted(lines.items()) for _ in range(q)]})
json.dump({'decks': decks, 'policies': ['bot', 'mix:0.2', 'mix:0.5']}, open(os.path.join(ROOT, 'decks/meta-tl/playable.corpus.json'), 'w'))
print('playable:', [d['name'] for d in decks], [len(d['cards']) for d in decks])
