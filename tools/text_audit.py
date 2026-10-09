"""Audit pool cards: official text (data/official_text.json) vs the printed
fields of the card in data/cards.json.

usage: text_audit.py [--all]   -> porting/text-audit.md (local), summary on stdout

Compares, for every pool row with official text: HP, type, attack names,
costs and base damage, ability names, weakness, resistance and retreat cost,
plus a word-overlap score of the rules text (low scores are listed for a
manual read). Card logic is not compared: a matching printed field says
nothing about the card's spec, so every flagged card still needs a look at it.
"""
import json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENERGY_LETTERS = set('grwlpfdmcn')
# Rules and reminder text that one side prints and the other omits.
REMINDERS = [
    r"you may play only 1 supporter card during your turn( \(before your attack\))?\.?",
    r"as long as this pokemon is on your bench, prevent all damage done to this pokemon by attacks \(both yours and your opponent's\) ?\.?",
    r"when your pokemon ex is knocked out, your opponent takes 2 prize cards\.?",
    r"you can't have more than 1 ace spec card in your deck\.?",
    r"you may play any number of item cards during your turn\.?",
    r"attach a pokemon tool to 1 of your pokemon that doesn't already have a pokemon tool attached\.?",
]


def norm(s):
    s = (s or '').lower().replace('pokémon', 'pokemon').replace('’', "'")
    for r in REMINDERS:
        s = re.sub(r, ' ', s)
    s = re.sub(r'\[[a-z]\]|\{[a-z]\}', ' ', s)
    return ' '.join(w for w in re.findall(r"[a-z0-9']+", s) if w not in ENERGY_LETTERS)


db = json.load(open(os.path.join(ROOT, 'data/official_text.json')))
tl = json.load(open(os.path.join(ROOT, 'data/cards.json')))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
TYPES = {'Grass': 1, 'Fire': 2, 'Water': 3, 'Lightning': 4, 'Psychic': 5, 'Fighting': 6, 'Darkness': 7,
         'Metal': 8, 'Colorless': 9, 'Fairy': 10, 'Dragon': 11}
LETTER = {'G': 1, 'R': 2, 'W': 3, 'L': 4, 'P': 5, 'F': 6, 'D': 7, 'M': 8, 'C': 9, 'Y': 10, 'N': 11}
COST = re.compile(r'^[GRWLPFDMCNY]+$|^0$')


def parse(t):
    lines = [l.strip() for l in t.split('\n') if l.strip()]
    o = {'hp': None, 'type': None, 'attacks': [], 'abilities': [], 'weakness': None, 'resistance': None, 'retreat': None, 'body': []}
    m = re.match(r'^- (\w+) - (\d+) HP', lines[1]) if len(lines) > 1 else None
    if m:
        o['type'], o['hp'] = TYPES.get(m.group(1)), int(m.group(2))
    skip = {i + 1 for i, l in enumerate(lines) if l.startswith('- Evolves from')}
    i = 2
    while i < len(lines):
        l = lines[i]
        if l == 'Ability:' and i + 1 < len(lines):
            o['abilities'].append(lines[i + 1])
            i += 2
            continue
        if COST.match(l) and i + 1 < len(lines):
            m = re.match(r'^(.*?)\s*(\d+)?([×x+\-])?$', lines[i + 1])
            cost = sorted(LETTER[c] for c in l if c in LETTER)
            o['attacks'].append({'name': m.group(1).strip(), 'damage': int(m.group(2) or 0), 'cost': cost})
            i += 2
            continue
        m = re.match(r'^Weakness: (\w+)', l)
        if m:
            o['weakness'] = TYPES.get(m.group(1))
        m = re.match(r'^Resistance: (\w+)', l)
        if m:
            o['resistance'] = TYPES.get(m.group(1))
        m = re.match(r'^Retreat: (\d+)', l)
        if m:
            o['retreat'] = int(m.group(1))
        if i not in skip and not re.match(r'^(Weakness|Resistance|Retreat|- |Pokémon|Trainer|Energy|Item|Supporter|Stadium|Pokémon Tool|ACE SPEC|Tera)', l):
            o['body'].append(l)
        i += 1
    return o


def nm(s):
    return re.sub(r"[^a-z0-9]", '', (s or '').lower().replace('é', 'e'))


def audit(row, o, c):
    out = []
    if c['superType'] == 1:
        if o['hp'] and o['hp'] != c.get('hp'):
            out.append('HP %s vs official %s' % (c.get('hp'), o['hp']))
        if o['type'] and o['type'] not in (c.get('cardType') or []):
            out.append('type %s vs official %s' % (c.get('cardType'), o['type']))
        w = [x['type'] for x in c.get('weakness') or []]
        if (o['weakness'] or None) != (w[0] if w else None):
            out.append('weakness %s vs official %s' % (w, o['weakness']))
        r = [x['type'] for x in c.get('resistance') or []]
        if (o['resistance'] or None) != (r[0] if r else None):
            out.append('resistance %s vs official %s' % (r, o['resistance']))
        if o['retreat'] is not None and o['retreat'] != len(c.get('retreat') or []):
            out.append('retreat %d vs official %d' % (len(c.get('retreat') or []), o['retreat']))
    ta = c.get('attacks') or []
    if [nm(a['name']) for a in o['attacks']] != [nm(a.get('name')) for a in ta]:
        out.append('attacks %s vs official %s' % ([a.get('name') for a in ta], [a['name'] for a in o['attacks']]))
    else:
        for a, b in zip(o['attacks'], ta):
            if a['damage'] != (b.get('damage') or 0):
                out.append('%s damage %s vs official %s' % (a['name'], b.get('damage'), a['damage']))
            if a['cost'] != sorted(x for x in b.get('cost') or [] if x):
                out.append('%s cost %s vs official %s' % (a['name'], b.get('cost'), a['cost']))
    tp = [p.get('name') for p in c.get('powers') or []]
    if [nm(x) for x in o['abilities']] != [nm(x) for x in tp]:
        out.append('abilities %s vs official %s' % (tp, o['abilities']))
    txt = ' '.join([c.get('text') or ''] + [a.get('text') or '' for a in ta] + [p.get('text') or '' for p in c.get('powers') or []])
    a, b = set(norm(' '.join(o['body'])).split()), set(norm(txt).split())
    ratio = len(a & b) / len(a | b) if (a or b) else 1.0
    return out, ratio


def main():
    rows = []
    for r in pool:
        key = '%s %s' % (r['set'], r['number'])
        if key not in db or r['key'] not in tl:
            continue
        o = parse(db[key]['text'])
        probs, ratio = audit(r, o, tl[r['key']])
        rows.append((r['key'], probs, ratio, db[key]['text']))
    flagged = [x for x in rows if x[1]]
    low = [x for x in rows if not x[1] and x[2] < 0.7]
    md = ['# Official text vs card data', '', 'Generated by `tools/text_audit.py`. %d pool rows with official text; '
          '%d with field mismatches, %d more with low text overlap (< 0.7).' % (len(rows), len(flagged), len(low)), '',
          '## Field mismatches', '']
    for k, probs, ratio, _ in flagged:
        md.append('- **%s**: %s' % (k, '; '.join(probs)))
    md += ['', '## Low text overlap (read the card)', '']
    for k, probs, ratio, t in sorted(low, key=lambda x: x[2]):
        md.append('- **%s**: overlap %.2f' % (k, ratio))
    open(os.path.join(ROOT, 'porting/text-audit.md'), 'w').write('\n'.join(md) + '\n')
    print('%d rows, %d field mismatches, %d low overlap -> porting/text-audit.md' % (len(rows), len(flagged), len(low)))
    if '--all' in sys.argv:
        for k, probs, ratio, _ in flagged:
            print(k, '|', '; '.join(probs))


if __name__ == '__main__':
    main()
