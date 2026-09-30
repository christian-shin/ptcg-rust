"""Write Twinleaf classes for vanilla pool printings (no abilities, no effect
text) straight from official card text, so the oracle can play them.

usage: gen_vanilla_twinleaf.py
Reads data/official_match.json (tools/compare_official.py) and
data/official_text.json; writes <set folder>/pool-additions.ts per set and
registers the classes in that set's index.ts (idempotent).
"""
import json, os, re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, 'twinleaf/ptcg-server/src')
match = json.load(open(os.path.join(ROOT, 'data/official_match.json')))
alias = json.load(open(os.path.join(ROOT, 'data/print_map.json')))['alias']
pool = {'%s %s' % (r['set'], r['number']): r for r in json.load(open(os.path.join(ROOT, 'data/pool.json')))}
dump_names = {c['fullName'] for c in json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))}
TYPE = {'Grass': 'G', 'Fire': 'R', 'Water': 'W', 'Lightning': 'L', 'Psychic': 'P', 'Fighting': 'F',
        'Darkness': 'D', 'Metal': 'M', 'Dragon': 'N', 'Colorless': 'C'}
STAGE = {'Basic': 'Stage.BASIC', 'Stage 1': 'Stage.STAGE_1', 'Stage 2': 'Stage.STAGE_2'}
COST = re.compile(r'^[GRWLPFDMCN]+$')


def parse(t):
    lines = [l.strip() for l in t.split('\n') if l.strip()]
    card = {'name': lines[0], 'attacks': [], 'weakness': [], 'resistance': [], 'retreat': 0, 'evolvesFrom': ''}
    m = re.search(r'- (\w+) - (\d+) HP', t)
    card['type'], card['hp'] = TYPE[m.group(1)], int(m.group(2))
    for i, l in enumerate(lines):
        if l.startswith('- ') and l[2:] in STAGE:
            card['stage'] = STAGE[l[2:]]
        if l.startswith('- Evolves from'):
            card['evolvesFrom'] = lines[i + 1]
        if COST.match(l) and i + 1 < len(lines):
            a = re.match(r'^(.*?)\s*(\d+)?$', lines[i + 1])
            card['attacks'].append({'name': a.group(1), 'cost': list(l), 'damage': int(a.group(2) or 0)})
        m = re.match(r'^Weakness: (\w+)', l)
        if m and m.group(1) in TYPE:
            card['weakness'] = [TYPE[m.group(1)]]
        m = re.match(r'^Resistance: (\w+) ?-?(\d+)?', l)
        if m and m.group(1) in TYPE:
            card['resistance'] = [(TYPE[m.group(1)], int(m.group(2) or 30))]
        m = re.match(r'^Retreat: (\d+)', l)
        if m:
            card['retreat'] = int(m.group(1))
    return card


def cls_name(name, set_code):
    return re.sub(r'[^A-Za-z0-9]', '', name.replace("'s", 's')) + set_code + 'Pool'


generated_before = set()
for root, _, files in os.walk(os.path.join(SRC, 'sets')):
    if 'pool-additions.ts' in files:
        generated_before |= set(re.findall(r"fullName: string = \"([^\"]+)\"", open(os.path.join(root, 'pool-additions.ts')).read()))
by_folder = {}
for key, r in match.items():
    o = r['official']
    # Vanilla printings Twinleaf lacks, plus ones this generator already wrote
    # (they now match their own generated class, which must be kept).
    ours = bool(r['best']) and r['best']['class'].endswith('Pool') and r['best']['fullName'] in generated_before
    if (r['verdict'].startswith('use') and not ours) or o['text'].strip() or o['abilities']:
        continue
    set_code, num = key.split(' ')
    row = pool[key]
    text = json.load(open(os.path.join(ROOT, 'data/official_text.json')))[key]['text']
    c = parse(text)
    full = '%s %s' % (c['name'], set_code)
    if full in dump_names and full not in generated_before:
        full = '%s %s %s' % (c['name'], set_code, num)
    folder = alias[set_code]['folders'][0]
    body = ['export class %s extends PokemonCard {' % cls_name(c['name'], set_code),
            '  public stage: Stage = %s;' % c['stage']]
    if c['evolvesFrom']:
        body.append("  public evolvesFrom = %s;" % json.dumps(c['evolvesFrom']))
    if c['name'].startswith("Team Rocket's"):
        body.append('  protected _tags = [CardTag.TEAM_ROCKET];')
    body += ['  public cardType: CardType[] = [%s];' % c['type'],
             '  public hp: number = %d;' % c['hp'],
             '  public weakness = [%s];' % ', '.join('{ type: %s }' % w for w in c['weakness']),
             '  public resistance = [%s];' % ', '.join('{ type: %s, value: -%d }' % x for x in c['resistance']),
             '  public retreat = [%s];' % ', '.join(['C'] * c['retreat']),
             '  public attacks = [%s];' % ', '.join("{ name: %s, cost: [%s], damage: %d, text: '' }" % (json.dumps(a['name']), ', '.join(a['cost']), a['damage']) for a in c['attacks']),
             "  public regulationMark = %s;" % json.dumps(row.get('reg') or ''),
             "  public set: string = '%s';" % set_code,
             "  public setNumber: string = '%s';" % num,
             "  public cardImage: string = 'assets/cardback.png';",
             '  public name: string = %s;' % json.dumps(c['name']),
             '  public fullName: string = %s;' % json.dumps(full),
             '}']
    by_folder.setdefault(folder, []).append((cls_name(c['name'], set_code), '\n'.join(body), full))

for folder, classes in by_folder.items():
    d = os.path.join(SRC, folder)
    header = ("// Generated by pkmntcg tools/gen_vanilla_twinleaf.py from official card text\n"
              "// (Limitless TCG): vanilla pool printings Twinleaf didn't have.\n"
              "import { PokemonCard } from '../../../game/store/card/pokemon-card';\n"
              "import { Stage, CardType%s } from '../../../game/store/card/card-types';\n\n"
              % (', CardTag' if any('CardTag.' in b for _, b, _ in classes) else ''))
    open(os.path.join(d, 'pool-additions.ts'), 'w').write(header + '\n\n'.join(b for _, b, _ in classes) + '\n')
    idx_path = os.path.join(d, 'index.ts')
    idx = open(idx_path).read()
    names = [n for n, _, _ in classes]
    imp = "import { %s } from './pool-additions';\n" % ', '.join(names)
    # Drop only this generator's previous registrations (other *Pool classes
    # registered by hand stay).
    m = re.search(r"import \{([^}]*)\} from './pool-additions';\n", idx)
    if m:
        for old_cls in [x.strip() for x in m.group(1).split(',') if x.strip()]:
            idx = idx.replace('  new %s(),\n' % old_cls, '')
        idx = idx.replace(m.group(0), '')
    idx = imp + idx
    # Register in the set's exported card array (the last `];` closes it).
    k = idx.rindex('];')
    idx = idx[:k] + ''.join('  new %s(),\n' % n for n in names) + idx[k:]
    open(idx_path, 'w').write(idx)
    for n, _, full in classes:
        print('%-40s %s -> %s' % (folder, n, full))
