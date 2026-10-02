"""Pre-evolutions the pool needs but doesn't contain -> data/support_cards.json.

Pool Pokémon such as Luxray ex or Seismitoad evolve from cards outside the
frozen pool (Shinx, Luxio, Tympole, Palpitoad...). Without them the
evolution can never reach play in a test game. For each missing name this
picks one Twinleaf printing, preferring one without card logic (no port
needed), then the evolved card's set, then the newest regulation mark, and
follows the chain down to the Basic. gen_carddb.py appends these to the
engine card DB; they are not pool cards.
"""
import collections, json, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
cards = json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
tl = {c['fullName']: c for c in cards}
byname = collections.defaultdict(list)
for c in cards:
    byname[c['name']].append(c)
LOGIC = {'reduceEffect', 'canPlay', 'canUseFromHandToBench'}


def vanilla(c):
    return not (set(c['$methods']) & LOGIC) and not any(
        isinstance(a, dict) and isinstance(a.get('effect'), dict) for a in (c.get('attacks') or []) + (c.get('powers') or []))


have = {tl[r['fullName']]['name'] for r in pool if r.get('fullName')}
out = []
todo = [tl[r['fullName']] for r in pool if r.get('fullName')]
while todo:
    c = todo.pop(0)
    ef = c.get('evolvesFrom')
    if not ef or ef in have:
        continue
    cands = byname.get(ef, [])
    if not cands:
        continue
    pick = max(cands, key=lambda x: (vanilla(x), x['set'] == c['set'], x.get('regulationMark') or '', x['fullName']))
    have.add(ef)
    out.append({'fullName': pick['fullName'], 'for': c['fullName'], 'vanilla': vanilla(pick)})
    todo.append(pick)
json.dump(out, open(os.path.join(ROOT, 'data/support_cards.json'), 'w'), indent=1, ensure_ascii=False)
for s in out:
    print('%-30s for %-28s %s' % (s['fullName'], s['for'], '' if s['vanilla'] else 'NEEDS PORT'))
print(len(out), 'support cards')
