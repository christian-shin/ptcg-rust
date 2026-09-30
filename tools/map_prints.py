"""Map every card-pool.tsv row to its exact Twinleaf printing.

Statically scans every card class under twinleaf/ptcg-server/src/sets
(registered in a set index.ts or not), resolves `extends` chains so a reprint
class inherits its base class's text and behavior, and matches each pool row
by (set code, printing number) through a derived set-code alias table, with
the card name as a sanity check.

Outputs
  data/print_map.json       one record per pool row
  data/print_map_report.md  rows whose mapping changes or is uncertain

Match quality
  exact               class with set == pool set code and number == pool number, registered
  exact-unregistered  same, but the class is not in the Twinleaf card dump / set index
  alias               set matched through an alias (folder or Twinleaf code differs), same number
  number-mismatch     same set (direct or alias) and same name, different number
  name-only           no class in the pool set; only same-named classes in other sets
  missing             no Twinleaf class with that name at all

Usage: python3 tools/map_prints.py [--root PKMNTCG_ROOT] [--src TWINLEAF_SRC] [--out DIR]
Read-only with respect to Twinleaf, data/pool.json and the engine. "current" is
the name-based mapping (card-pool.tsv), also after tools/pool_meta.py has
applied this map to pool.json (it keeps the old printing as `prev_fullName`);
`pool_fullName` is the printing pool.json uses now.
"""
import argparse, collections, csv, difflib, json, os, re, unicodedata

ap = argparse.ArgumentParser()
ap.add_argument('--root', default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ap.add_argument('--src', default=None, help='Twinleaf ptcg-server/src (default ROOT/twinleaf/ptcg-server/src)')
ap.add_argument('--out', default=None, help='output dir (default ROOT/data)')
args = ap.parse_args()
ROOT = args.root
SRC = args.src or os.path.join(ROOT, 'twinleaf/ptcg-server/src')
SETS = os.path.join(SRC, 'sets')
OUT = args.out or os.path.join(ROOT, 'data')

LOGIC = {'reduceEffect', 'canPlay', 'canUseFromHandToBench'}
PRINT_FIELDS = {'set', 'setNumber', 'fullName', 'name', 'cardImage', 'regulationMark', 'legacyFullName',
                'illustrator', 'id', 'artist'}
CONTENT_FIELDS = ['stage', 'evolvesFrom', 'cardType', 'superType', 'hp', 'weakness', 'resistance', 'retreat',
                  'powers', 'attacks', 'trainerType', 'energyType', 'provides', 'text', 'tags', 'cardTag']

# ---------------------------------------------------------------- TS scanning


def skip_str(s, i):
    q, i = s[i], i + 1
    while i < len(s):
        if s[i] == '\\':
            i += 2
            continue
        if s[i] == q:
            return i + 1
        if s[i] == '\n' and q != '`':
            return i + 1
        if q == '`' and s.startswith('${', i):
            i = match(s, i + 1)
            continue
        i += 1
    return i


def skip_trivia(s, i):
    while i < len(s):
        if s[i].isspace():
            i += 1
        elif s.startswith('//', i):
            j = s.find('\n', i)
            i = len(s) if j < 0 else j
        elif s.startswith('/*', i):
            j = s.find('*/', i + 2)
            i = len(s) if j < 0 else j + 2
        else:
            break
    return i


def match(s, i):
    """s[i] is an opening bracket; return the index just past its partner."""
    depth = 0
    while i < len(s):
        c = s[i]
        if c in '\'"`':
            i = skip_str(s, i)
            continue
        if s.startswith('//', i) or s.startswith('/*', i):
            i = skip_trivia(s, i)
            continue
        if c in '{([':
            depth += 1
        elif c in '})]':
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return i


NEXT_MEMBER = re.compile(r'\n\s*(public|private|protected|readonly|static|override|async|get\s|set\s|constructor|\w+\s*[(:=?!])')


def members(body):
    """Top-level class members -> (fields {name: raw value}, methods set)."""
    fields, methods, i, n = {}, set(), 0, len(body)
    while True:
        i = skip_trivia(body, i)
        if i >= n:
            break
        start, name, kind = i, None, None
        while i < n:
            c = body[i]
            if c in '\'"`':
                i = skip_str(body, i)
                continue
            if c == '(' and kind is None:
                kind = 'method'
                pre = re.findall(r'(\w+)\s*(?:<[^>]*>)?\s*$', body[start:i])
                name = pre[0] if pre else '?'
                i = match(body, i)
                j = i
                while j < n and body[j] not in '{;':
                    if body[j] in '([':
                        j = match(body, j)
                        continue
                    j += 1
                i = match(body, j) if j < n and body[j] == '{' else j + 1
                methods.add(name)
                break
            if c == '=' and kind is None and body[i + 1:i + 2] not in ('=', '>'):
                kind = 'field'
                pre = re.findall(r'(\w+)\s*[?!]?\s*(?::[^=]*)?$', body[start:i])
                name = pre[0] if pre else '?'
                vstart = i + 1
                i += 1
                while i < n:
                    c = body[i]
                    if c in '\'"`':
                        i = skip_str(body, i)
                        continue
                    if body.startswith('//', i) or body.startswith('/*', i):
                        i = skip_trivia(body, i)
                        continue
                    if c in '{([':
                        i = match(body, i)
                        continue
                    if c == ';':
                        break
                    if c == '\n' and NEXT_MEMBER.match(body, i) and body[vstart:i].strip() and \
                            body[vstart:i].rstrip()[-1] not in '+,=?:|&(':
                        break
                    i += 1
                fields[name] = body[vstart:i].strip()
                i += 1
                break
            if c == ';' or c == '\n' and kind is None and body[start:i].strip() and NEXT_MEMBER.match(body, i):
                i += 1  # declaration without initializer
                break
            if c == '{':  # stray block
                i = match(body, i)
                break
            i += 1
    return fields, methods


STR_RE = re.compile(r"""^(['"`])(.*)\1$""", re.S)


def lit(raw):
    """Value of a plain string literal (or concatenation of them), else None."""
    if raw is None:
        return None
    parts = re.split(r"""(?<=['"`])\s*\+\s*(?=['"`])""", raw.strip())
    out = []
    for p in parts:
        m = STR_RE.match(p.strip())
        if not m:
            return None
        v = m.group(2)
        v = re.sub(r'\\u([0-9a-fA-F]{4})', lambda x: chr(int(x.group(1), 16)), v)
        v = re.sub(r'\\n', '\n', v)
        v = re.sub(r'\\(.)', r'\1', v)
        out.append(v)
    return ''.join(out)


CLASS_RE = re.compile(r'^[ \t]*(export\s+)?(?:abstract\s+)?class\s+(\w+)(?:<[^>{]*>)?\s+extends\s+([\w.]+)[^{]*\{', re.M)
IMPORT_RE = re.compile(r'import\s*\{([^}]*)\}\s*from\s*[\'"]([^\'"]+)[\'"]', re.S)


def resolve_mod(frm, mod):
    if not mod.startswith('.'):
        return None
    p = os.path.normpath(os.path.join(os.path.dirname(frm), mod))
    for c in (p + '.ts', os.path.join(p, 'index.ts')):
        if os.path.exists(c):
            return c
    return p + '.ts'


def parse_imports(path, src):
    imp = {}
    for names, mod in IMPORT_RE.findall(src):
        tgt = resolve_mod(path, mod)
        names = re.sub(r'//[^\n]*|/\*.*?\*/', '', names, flags=re.S)
        for n in names.split(','):
            m = re.match(r'\s*(?:type\s+)?(\w+)(?:\s+as\s+(\w+))?', n)
            if not m:
                continue
            imp[m.group(2) or m.group(1)] = (tgt, m.group(1))
    return imp


def rel(p):
    return os.path.relpath(p, SRC) if p else p


def set_folder(path):
    d = os.path.dirname(path)
    while d.startswith(SETS) and d != SETS:
        if os.path.exists(os.path.join(d, 'index.ts')):
            return d
        d = os.path.dirname(d)
    return os.path.dirname(path)


classes = {}      # (file, cls) -> record
imports = {}      # file -> imports
by_file = collections.defaultdict(dict)
for dp, dn, fn in os.walk(SETS):
    dn[:] = [d for d in dn if d != 'tests']
    for f in fn:
        if not f.endswith('.ts') or f.startswith('generate-') or f.endswith('.spec.ts'):
            continue
        path = os.path.join(dp, f)
        src = open(path, encoding='utf-8').read()
        imports[path] = parse_imports(path, src)
        for m in CLASS_RE.finditer(src):
            b = m.end() - 1
            e = match(src, b)
            fields, methods = members(src[b + 1:e - 1])
            body = src[b + 1:e - 1]
            logic = re.sub(r'\s+', '', re.sub(r'//[^\n]*', '', body))
            for k in PRINT_FIELDS:
                if k in fields:
                    logic = logic.replace(re.sub(r'\s+', '', fields[k]), '')
            rec = dict(file=path, cls=m.group(2), extends=m.group(3), exported=bool(m.group(1)),
                       fields=fields, methods=methods, folder=set_folder(path), logic=logic)
            classes[(path, m.group(2))] = rec
            by_file[path][m.group(2)] = rec


def parent_of(rec):
    ext = rec['extends']
    if ext in by_file[rec['file']] and ext != rec['cls']:
        return by_file[rec['file']][ext]
    imp = imports[rec['file']].get(ext)
    if imp and imp[0] and (imp[0], imp[1]) in classes:
        return classes[(imp[0], imp[1])]
    return None


def chain(rec):
    out, seen = [], set()
    while rec and (rec['file'], rec['cls']) not in seen:
        seen.add((rec['file'], rec['cls']))
        out.append(rec)
        rec = parent_of(rec)
    return out


def norm_raw(v):
    v = re.sub(r"""(['"`])\s*\+\s*(['"`])""", '', v)
    v = v.replace('"', "'").replace("\\'", "'")
    return re.sub(r'\s+', '', v).rstrip(',')


for rec in classes.values():
    ch = chain(rec)
    res = {}
    for r in reversed(ch):
        res.update(r['fields'])
    rec['chain'] = ch
    rec['resolved'] = res
    rec['base'] = ch[-1]['extends'] if ch else rec['extends']
    beh = next((r for r in ch if LOGIC & r['methods']), None)
    rec['behavior'] = (beh['file'], beh['cls']) if beh else None
    content = next((r for r in ch if set(r['fields']) - PRINT_FIELDS), ch[0])
    rec['content_class'] = (content['file'], content['cls'])
    rec['sig'] = '|'.join('%s=%s' % (k, norm_raw(res[k])) for k in CONTENT_FIELDS if k in res)
    rec['name'] = lit(res.get('name'))
    rec['set'] = lit(res.get('set'))
    rec['setNumber'] = lit(res.get('setNumber'))
    rec['fullName'] = lit(res.get('fullName'))
    rec['is_card'] = rec['base'] in ('PokemonCard', 'TrainerCard', 'EnergyCard') and rec['fullName'] is not None

cards = [r for r in classes.values() if r['is_card']]


def readable(rec):
    res = rec['resolved']
    out = []
    if 'hp' in res:
        out.append('HP %s' % res['hp'])
    for key in ('powers', 'attacks'):
        v = res.get(key)
        if not v:
            continue
        for m in re.finditer(r"""\b(name|damage|text)\s*:\s*((?:(['"`])(?:\\.|(?!\3).)*\3\s*\+?\s*)+|[\w.'"]+)""", v, re.S):
            val = lit(m.group(2).strip().rstrip('+').strip()) if m.group(1) != 'damage' else m.group(2)
            val = val if val is not None else m.group(2)
            out.append(('[%s] ' % val) if m.group(1) == 'name' else ('%s: %s' % (m.group(1), val)))
    t = lit(res.get('text')) if res.get('text') else None
    if t:
        out.append('text: ' + t)
    return re.sub(r'\s+', ' ', ' '.join(out)).strip()


# ------------------------------------------------------------- registration
static_reg = set()   # (file, cls) registered via some set index.ts
reg_index = {}
for dp, dn, fn in os.walk(SETS):
    if 'index.ts' in fn:
        p = os.path.join(dp, 'index.ts')
        src = open(p, encoding='utf-8').read()
        imp = parse_imports(p, src)
        local = {r['cls']: r for r in by_file.get(p, {}).values()}
        for n in re.findall(r'new\s+(\w+)\s*\(', src):
            if n in local:
                key = (p, n)
            elif n in imp:
                key = (imp[n][0], imp[n][1])
            else:
                continue
            static_reg.add(key)
            reg_index.setdefault(key, p)

dump = json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))
dump_by_full = {c['fullName']: c for c in dump}
for r in cards:
    r['registered_static'] = (r['file'], r['cls']) in static_reg
    d = dump_by_full.get(r['fullName'])
    r['in_dump'] = bool(d and d['$class'] == r['cls'] and str(d.get('setNumber')) == str(r['setNumber']))
    r['registered'] = r['in_dump'] or r['registered_static']

# parser sanity vs dump
checked = bad = 0
for r in cards:
    if r['in_dump']:
        d = dump_by_full[r['fullName']]
        checked += 1
        if d.get('name') != r['name'] or str(d.get('set')) != str(r['set']):
            bad += 1
print('static scan: %d classes, %d cards; %d checked against dump, %d disagree' % (len(classes), len(cards), checked, bad))

# ----------------------------------------------------------------- aliases


ENERGY_SYM = dict(G='Grass', R='Fire', W='Water', L='Lightning', P='Psychic', F='Fighting', D='Darkness',
                  M='Metal', C='Colorless', N='Dragon', Y='Fairy')


def nname(s):
    s = re.sub(r'\[([A-Z])\]', lambda m: ENERGY_SYM.get(m.group(1), m.group(0)), s or '')
    s = unicodedata.normalize('NFKD', s).replace('’', "'")
    s = ''.join(c for c in s if not unicodedata.combining(c)).lower()
    return re.sub(r'[^a-z0-9]', '', s)


def nnum(s):
    s = (s or '').strip().lower()
    return s.lstrip('0') or s


pool_rows = list(csv.DictReader(open(os.path.join(ROOT, 'card-pool.tsv'), encoding='utf-8'), delimiter='\t'))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
assert len(pool) == len(pool_rows)
pool_codes = sorted({r['set'] for r in pool_rows})

folder_codes = collections.defaultdict(collections.Counter)
for r in cards:
    folder_codes[r['folder']][r['set']] += 1
code_folders = collections.defaultdict(set)   # twinleaf set code -> folders where it is the majority code
for f, cnt in folder_codes.items():
    code_folders[cnt.most_common(1)[0][0]].add(f)

alias = {}
for code in pool_codes:
    folders = set(code_folders.get(code, set())) | {f for f, cnt in folder_codes.items() if cnt[code] >= 10}
    tl_codes = {code}
    for f in folders:
        # secondary codes that are the folder's own printings (not reprints of another set's code)
        for c, n in folder_codes[f].items():
            if c not in code_folders or c == code:
                tl_codes.add(c)
    alias[code] = dict(folders=sorted(rel(f) for f in folders), twinleaf_codes=sorted(tl_codes))

by_setnum = collections.defaultdict(list)
by_name = collections.defaultdict(list)
for r in cards:
    by_setnum[(r['set'], nnum(r['setNumber']))].append(r)
    by_name[nname(r['name'])].append(r)


def in_pool_set(r, code):
    a = alias[code]
    return r['set'] in a['twinleaf_codes'] or rel(r['folder']) in a['folders']


def pick(cands):
    return sorted(cands, key=lambda r: (not r['registered'], rel(r['file']), r['cls']))[0] if cands else None


set_cards = {c: [r for r in cards if in_pool_set(r, c)] for c in pool_codes}

# ------------------------------------------------------------------- ports
ports = []
IMPLS = os.path.join(ROOT, 'engine/src/cards/impls')
for f in sorted(os.listdir(IMPLS)) if os.path.isdir(IMPLS) else []:
    if f.endswith('.rs'):
        for k in re.findall(r'class:\s*"([^"]+)"', open(os.path.join(IMPLS, f), encoding='utf-8').read()):
            ports.append((k, 'engine/src/cards/impls/' + f))


def ports_for(beh_cls, set_code, full):
    out = []
    for k, f in ports:
        c, _, q = k.partition('@')
        if c == beh_cls and (not q or any(x in (set_code, full) for x in q.split('|'))):
            out.append(dict(key=k, file=f))
    return out


def cref(r):
    if not r:
        return None
    return dict(cls=r['cls'], fullName=r['fullName'], set=r['set'], number=r['setNumber'], name=r['name'],
                file=rel(r['file']), registered=r['registered'], in_dump=r['in_dump'],
                registered_static=r['registered_static'], extends=r['extends'],
                base_chain=[x['cls'] + ' (' + rel(x['file']) + ')' for x in r['chain'][1:]],
                behavior=('%s (%s)' % (r['behavior'][1], rel(r['behavior'][0]))) if r['behavior'] else None,
                content_class='%s (%s)' % (r['content_class'][1], rel(r['content_class'][0])), text=readable(r))


full_index = collections.defaultdict(list)
for r in cards:
    full_index[r['fullName']].append(r)


def old_class(p):
    # `current` is the name-based mapping: card-pool.tsv's, i.e. pool.json's
    # `prev_fullName` for rows pool_meta.py already remapped to this map.
    fl = p['prev_fullName'] if 'prev_fullName' in p else p.get('fullName')
    if not fl:
        return None
    cands = full_index.get(fl, [])
    tf = os.path.join(SRC, p['twinleaf_file']) if p.get('twinleaf_file') and 'prev_fullName' not in p else None
    return pick([c for c in cands if c['in_dump']] or [c for c in cands if c['file'] == tf] or cands)


# Manual review of exact matches whose Twinleaf reprint class inherits text that
# does not look like the printing's official text (checked by hand, keep short).
REVIEW = {
    ('MEG', '125'): 'Twinleaf RareCandyMEG extends the ex Holon Phantoms RareCandy (old "Stage 1 or Stage 2" '
                    'text); the MEG printing has the modern SVI text. Keep the SVI behavior; Twinleaf should '
                    'make RareCandyMEG extend the SVI RareCandy.',
    ('ASC', '60'): 'Twinleaf EelektrikASC extends the Noble Victories Eelektrik ("before your attack" wording); '
                   'the BLK Eelektrik the pool maps now has the modern wording. Verify against the official ASC '
                   'text before switching; Twinleaf may need EelektrikASC to extend the BLK class.',
}

# ----------------------------------------------------------------- matching
results = []
for row, p in zip(pool_rows, pool):
    code, num, nm = row['set'], nnum(row['number']), nname(row['name'])
    same_name = by_name.get(nm, [])
    exact = [r for r in by_setnum.get((code, num), []) if nname(r['name']) == nm]
    aliased = [r for r in set_cards[code] if nnum(r['setNumber']) == num and nname(r['name']) == nm]
    fuzzy = ''
    if not exact and not aliased:
        # same printing slot, name spelled differently in Twinleaf (typo / romanization)
        sim = lambda r: difflib.SequenceMatcher(None, nname(r['name']), nm).ratio() >= 0.8
        exact = [r for r in by_setnum.get((code, num), []) if sim(r)]
        aliased = [] if exact else [r for r in set_cards[code] if nnum(r['setNumber']) == num and sim(r)]
        if exact or aliased:
            fuzzy = 'Twinleaf spells the name %r' % (exact or aliased)[0]['name']
    in_set_named = [r for r in same_name if in_pool_set(r, code)]
    note = fuzzy
    if exact:
        m = pick(exact)
        q = 'exact' if m['registered'] else 'exact-unregistered'
    elif aliased:
        m, q = pick(aliased), 'alias'
    elif in_set_named:
        m, q = pick(in_set_named), 'number-mismatch'
        if len(in_set_named) > 1:
            note = '%d same-named classes in set: %s' % (len(in_set_named), ', '.join(
                '%s #%s' % (r['cls'], r['setNumber']) for r in in_set_named))
    elif same_name:
        m, q = None, 'name-only'
    else:
        m, q = None, 'missing'
    occupied = [r for r in by_setnum.get((code, num), []) if nname(r['name']) != nm]
    if occupied and q not in ('exact', 'exact-unregistered'):
        note = (note + '; ' if note else '') + 'number %s %s holds %s' % (code, row['number'], ', '.join(
            '%s (%s)' % (r['name'], r['cls']) for r in occupied))

    old = old_class(p)
    new = m or None
    changed = bool(new and (not old or new['fullName'] != old['fullName'] or new['file'] != old['file']))
    if q in ('exact', 'exact-unregistered', 'alias'):
        conf = 'high'
    elif q == 'number-mismatch' and new and ((old and old['sig'] == new['sig']) or
                                             (len(new['chain']) > 1 and new['set'] == code)):
        conf = 'medium'   # same text as current, or a reprint subclass stamped with the pool set code
    else:
        conf = 'low'
    text_diff = bool(new and old and new['sig'] != old['sig'])
    rec = dict(set=row['set'], number=row['number'], name=row['name'], status=row['status'], match=q, confidence=conf, note=note,
               current=dict(fullName=p.get('fullName'), file=p.get('twinleaf_file') or None,
                            set=p.get('twinleaf_set') or None, number=p.get('twinleaf_number') or None,
                            cls=p.get('cls')) if 'prev_fullName' not in p else
                       dict(fullName=old['fullName'], file=rel(old['file']), set=old['set'], number=old['setNumber'], cls=old['cls'])
                       if old else dict(fullName=None, file=None, set=None, number=None, cls=None),
               pool_fullName=p.get('fullName'),
               current_class=cref(old), twinleaf=cref(new), changes=changed, text_differs=text_diff,
               review=REVIEW.get((row['set'], row['number'])))
    if q in ('name-only', 'missing', 'number-mismatch') or (new and not new['registered']):
        pool_cands = same_name if q != 'missing' else [r for r in cards if nm and (nm in nname(r['name']) or nname(r['name']) in nm) and len(nname(r['name'])) > 3]
        groups = collections.OrderedDict()
        for r in sorted(pool_cands, key=lambda r: (rel(r['file']), r['cls'])):
            groups.setdefault(r['sig'], []).append(r)
        rec['candidates'] = [dict(text=readable(g[0]), same_as_current=bool(old and old['sig'] == s),
                                  classes=['%s | %s | %s #%s | %s%s' % (r['cls'], r['fullName'], r['set'], r['setNumber'],
                                                                         rel(r['file']), '' if r['registered'] else ' | UNREGISTERED')
                                           for r in g]) for s, g in groups.items()]
    # port cross-check
    if changed:
        old_beh = old['behavior'] if old else None
        new_beh = new['behavior']
        old_ports = ports_for(old_beh[1], old['set'], old['fullName']) if old_beh else []
        new_ports = ports_for(new_beh[1], new['set'], new['fullName']) if new_beh else []
        if not old:
            verdict = 'newly mapped (was unmapped); ' + ('no logic' if not new_beh else
                                                          'ports matching: ' + (', '.join(x['key'] for x in new_ports) or 'none, port needed'))
        elif not new_beh:
            verdict = 'no logic at new printing' + ('; drop binding of old port' if old_ports else '')
        elif old_beh == new_beh:
            verdict = 'rebind (same behavior class)'
            if old_ports and not new_ports:
                verdict += '; port key is qualified to the old printing, requalify it'
            elif not old_ports:
                verdict += '; no port yet'
        elif old_beh and classes[old_beh]['logic'] == classes[new_beh]['logic']:
            verdict = 'rebind (behavior class in another file, but its source is identical)'
            if old_ports and not new_ports:
                verdict += '; port key is qualified to the old printing, requalify it'
        else:
            verdict = 're-port (different behavior class)'
            if not text_diff:
                verdict += ', printed text same: diff the two implementations, re-port only if behavior differs'
            if not old_ports and not new_ports:
                verdict += '; no port exists yet'
            elif any(x['key'] == y['key'] for x in old_ports for y in new_ports):
                verdict += '; WARNING existing port key would silently bind to the new class with old behavior'
            elif new_ports:
                verdict += '; a port matching the new key exists (%s), verify it ports the new class' % ', '.join(x['key'] for x in new_ports)
        rec['port'] = dict(old_behavior=('%s (%s)' % (old_beh[1], rel(old_beh[0]))) if old_beh else None,
                           new_behavior=('%s (%s)' % (new_beh[1], rel(new_beh[0]))) if new_beh else None,
                           old_ports=old_ports, new_ports=new_ports, verdict=verdict)
    if new and not new['registered']:
        idx = os.path.join(new['folder'], 'index.ts')
        rec['register'] = dict(index=rel(idx), cls=new['cls'], defined_in=rel(new['file']), exported=next(
            (x['exported'] for x in [classes[(new['file'], new['cls'])]]), True))
    results.append(rec)

os.makedirs(OUT, exist_ok=True)
json.dump(dict(alias=alias, results=results), open(os.path.join(OUT, 'print_map.json'), 'w'), indent=1, ensure_ascii=False)

# ------------------------------------------------------------------ report
Q = ['exact', 'exact-unregistered', 'alias', 'number-mismatch', 'name-only', 'missing']
cnt = collections.Counter(r['match'] for r in results)
L = ['# Pool print map', '', 'Generated by `tools/map_prints.py`. Pool rows: %d. Twinleaf card classes scanned: %d '
     '(%d registered).' % (len(results), len(cards), sum(r['registered'] for r in cards)), '']
L += ['| match | rows |', '|---|---|'] + ['| %s | %d |' % (q, cnt[q]) for q in Q] + ['']
L += ['Changed mappings: %d (text differs in %d).' % (sum(r['changes'] for r in results),
                                                     sum(r['changes'] and r['text_differs'] for r in results)), '']
L += ['## Set-code aliases', '', '| pool code | Twinleaf set codes | folders |', '|---|---|---|']
for c in pool_codes:
    a = alias[c]
    L.append('| %s | %s | %s |' % (c, ', '.join(a['twinleaf_codes']), ', '.join(a['folders']) or '(none)'))
L.append('')


def row_line(r):
    t, cur = r['twinleaf'], r['current']
    s = '- **%s %s %s** (%s, confidence %s): current `%s`' % (r['set'], r['number'], r['name'], r['status'],
                                                            r['confidence'], cur['fullName'])
    if t:
        s += ' -> `%s` (`%s`, set %s #%s, %s%s)' % (t['fullName'], t['cls'], t['set'], t['number'], t['file'],
                                                   '' if t['registered'] else ', UNREGISTERED')
        if t['base_chain']:
            s += ', extends %s' % ' < '.join(t['base_chain'])
    if r['changes']:
        s += '; text %s' % ('n/a (was unmapped)' if not r['current_class'] else 'DIFFERS' if r['text_differs'] else 'same')
    if r.get('note'):
        s += '. Note: ' + r['note']
    out = [s]
    if r.get('review'):
        out.append('  - REVIEW: ' + r['review'])
    if r['changes'] and r['text_differs']:
        out.append('  - old text: ' + r['current_class']['text'][:600])
        out.append('  - new text: ' + t['text'][:600])
    if r.get('port'):
        pr = r['port']
        out.append('  - port: %s. old behavior %s, new behavior %s; old ports %s; new-key ports %s' % (
            pr['verdict'], pr['old_behavior'], pr['new_behavior'],
            ', '.join('`%s` (%s)' % (x['key'], x['file']) for x in pr['old_ports']) or 'none',
            ', '.join('`%s`' % x['key'] for x in pr['new_ports']) or 'none'))
    if r.get('register'):
        out.append('  - register: add `new %s()` to %s (class in %s%s)' % (
            r['register']['cls'], r['register']['index'], r['register']['defined_in'],
            '' if r['register']['exported'] else ', not exported: add `export`'))
    for c in r.get('candidates', []):
        out.append('  - candidate%s: %s' % (' (same text as current)' if c['same_as_current'] else '', c['text'][:700] or '(no text)'))
        for k in c['classes']:
            out.append('    - `%s`' % k)
    return out




def plain(r):
    v = r.get('port', {}).get('verdict', '')
    return not r['text_differs'] and not r.get('review') and not r.get('register') and r['confidence'] == 'high' and \
        (v.startswith('rebind (same behavior class)') and 'requalify' not in v or v == 'no logic at new printing')


L += ['## Action needed', '',
      'Changed rows where the printed text differs, a port must be written/re-ported/requalified, '
      'the class is unregistered, or the match is not an exact printing.', '']
for q in Q:
    rows = [r for r in results if r['changes'] and r['match'] == q and not plain(r)]
    if rows:
        L += ['### %s (%d)' % (q, len(rows)), '']
        for r in rows:
            L += row_line(r)
        L.append('')
L += ['## Plain rebinds', '',
      'Exact printing found, same printed text and same behavior class as the current mapping: only '
      '`fullName` changes (existing unqualified port keys keep working).', '']
rows = [r for r in results if r['changes'] and plain(r)]
L += ['| pool row | current | new | class | port |', '|---|---|---|---|---|']
for r in rows:
    t = r['twinleaf']
    L.append('| %s %s %s | %s | %s | %s | %s |' % (r['set'], r['number'], r['name'], r['current']['fullName'], t['fullName'],
                                               t['cls'], ', '.join(x['key'] for x in r['port']['new_ports']) or '-'))
L.append('')
L += ['## Uncertain rows (mapping unchanged)', '']
for q in ['exact-unregistered', 'alias', 'number-mismatch', 'name-only', 'missing']:
    rows = [r for r in results if not r['changes'] and r['match'] == q]
    if rows:
        L += ['### %s (%d)' % (q, len(rows)), '']
        for r in rows:
            L += row_line(r)
        L.append('')
regs = collections.defaultdict(list)
for r in results:
    if r.get('register') and r['changes']:
        regs[r['register']['index']].append(r['register']['cls'])
L += ['## Twinleaf index registrations needed', '']
for k in sorted(regs):
    L.append('- %s: %s' % (k, ', '.join(sorted(set(regs[k])))))
L.append('')
open(os.path.join(OUT, 'print_map_report.md'), 'w').write('\n'.join(L))
print(dict(cnt), 'changed:', sum(r['changes'] for r in results))
