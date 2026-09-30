"""Join card-pool.tsv with the Twinleaf card dump -> data/pool.json.

Each pool row gets the Twinleaf card record (fullName, class, methods) found
by (twinleaf_set, twinleaf_number), and a tier guess:
  data    - no reduceEffect (printed data only)
  short   - reduceEffect / effect bodies short enough to be prefab calls
  custom  - everything else

Printings: card-pool.tsv's twinleaf_* columns were chosen by card name. When
data/print_map.json (tools/map_prints.py) exists, rows whose exact printing it
found are remapped to that printing: match `exact` / `exact-unregistered`, and
`number-mismatch` with medium confidence (same text as the name-based mapping,
or a reprint class stamped with the pool set code). Rows with a manual REVIEW
note (Twinleaf's reprint class may inherit the wrong text) and name-only /
missing / low-confidence rows keep the name-based mapping. Every row gets
`print_match` and `print_note`; remapped rows also get `prev_fullName` (the
name-based printing, null if the row was unmapped), which gen_carddb.py keeps in the engine card DB so
existing decks and traces still load.
"""
import csv, json, os, re, collections
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, 'twinleaf/ptcg-server/src')
cards = json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))
by_key = collections.defaultdict(list)
for c in cards:
    by_key[(c.get('set'), str(c.get('setNumber')))].append(c)

def reduce_lines(src, cls):
    """Lines in the class's reduceEffect body (0 if none)."""
    m = re.search(r'class %s\b' % re.escape(cls), src)
    body = src[m.start():] if m else src
    m = re.search(r'public reduceEffect\([^)]*\)[^{]*\{', body)
    if not m:
        return 0
    i, depth = m.end(), 1
    while i < len(body) and depth:
        depth += {'{': 1, '}': -1}.get(body[i], 0)
        i += 1
    return body[m.end():i].count('\n')

dump_by_full = {c['fullName']: c for c in cards}
PM_PATH = os.path.join(ROOT, 'data/print_map.json')
print_map = {}
if os.path.exists(PM_PATH):
    print_map = {(m['set'], m['number']): m for m in json.load(open(PM_PATH))['results']}


OM_PATH = os.path.join(ROOT, 'data/official_match.json')
official = json.load(open(OM_PATH)) if os.path.exists(OM_PATH) else {}


def accept(m):
    """Remap this row to print_map's printing?"""
    if not m or not m.get('twinleaf') or m.get('review'):
        return False
    if m['match'] in ('exact', 'exact-unregistered'):
        return True
    return m['match'] == 'number-mismatch' and m['confidence'] == 'medium'


def src_of(ref):
    """'Cls (sets/x.ts)' -> source path relative to SRC."""
    return ref[ref.rindex('(') + 1:-1] if ref else None


def print_note(m, rec, prev, remapped):
    if not m:
        return 'no print_map.json record'
    parts = []
    if remapped and prev != rec.get('fullName'):
        parts.append('remapped from %s to %s (%s #%s)' % (prev or 'unmapped', rec['fullName'], m['twinleaf']['set'],
                                                         m['twinleaf']['number']))
        if m.get('text_differs'):
            parts.append('printed text differs from the old printing')
    elif remapped:
        parts.append('exact printing')
    else:
        why = {'name-only': 'no Twinleaf class in the pool set',
               'missing': 'no Twinleaf class with this name',
               'number-mismatch': 'same-named class in set at another number, low confidence'}.get(m['match'], 'doubtful')
        parts.append('%s; kept name-based mapping %s; needs official card text' % (why, rec.get('fullName') or '(none)'))
        if m.get('twinleaf') and m['twinleaf']['fullName'] != rec.get('fullName'):
            parts.append('candidate %s (%s #%s)' % (m['twinleaf']['fullName'], m['twinleaf']['set'], m['twinleaf']['number']))
    if m.get('review'):
        parts.append('REVIEW: ' + m['review'])
    if m.get('note'):
        parts.append(m['note'])
    return '; '.join(parts)


rows = list(csv.DictReader(open(os.path.join(ROOT, 'card-pool.tsv')), delimiter='\t'))
out = []
for r in rows:
    rec = dict(r)
    if r['twinleaf_file']:
        cands = by_key.get((r['twinleaf_set'], r['twinleaf_number']), [])
        if len(cands) > 1:
            src = open(os.path.join(SRC, r['twinleaf_file'])).read()
            in_file = set(re.findall(r'export class (\w+)', src))
            cands = [c for c in cands if c['$class'] in in_file] or cands
        if len(cands) != 1:
            rec['error'] = 'matches=%d' % len(cands)
        else:
            c = cands[0]
            src = open(os.path.join(SRC, r['twinleaf_file'])).read()
            n = reduce_lines(src, c['$class'])
            fn_fields = sum(1 for a in (c.get('attacks') or []) + (c.get('powers') or []) if isinstance(a, dict) and isinstance(a.get('effect'), dict))
            has_logic = 'reduceEffect' in c['$methods'] or fn_fields > 0 or any(m in c['$methods'] for m in ('canPlay', 'canUseFromHandToBench'))
            rec.update(fullName=c['fullName'], cls=c['$class'], methods=c['$methods'], reduce_lines=n, effect_fns=fn_fields,
                       tier='data' if not has_logic else ('short' if n <= 15 else 'custom'))
    m = print_map.get((r['set'], r['number']))
    prev = rec.get('fullName')
    remapped = accept(m)
    if remapped and m['twinleaf']['fullName'] != prev:
        t = m['twinleaf']
        c = dump_by_full.get(t['fullName'])
        if not c or c['$class'] != t['cls']:
            raise SystemExit('print_map printing %s (%s) is not in the Twinleaf dump; register it and re-dump' % (t['fullName'], t['cls']))
        beh = src_of(t.get('behavior'))
        n = reduce_lines(open(os.path.join(SRC, beh)).read(), t['behavior'].split(' ')[0]) if beh else 0
        fn_fields = sum(1 for a in (c.get('attacks') or []) + (c.get('powers') or []) if isinstance(a, dict) and isinstance(a.get('effect'), dict))
        has_logic = 'reduceEffect' in c['$methods'] or fn_fields > 0 or any(x in c['$methods'] for x in ('canPlay', 'canUseFromHandToBench'))
        rec.pop('error', None)
        rec.update(twinleaf_file=t['file'], twinleaf_set=str(c['set']),
                   twinleaf_number=str(c['setNumber']), fullName=c['fullName'], cls=c['$class'], methods=c['$methods'],
                   reduce_lines=n, effect_fns=fn_fields, tier='data' if not has_logic else ('short' if n <= 15 else 'custom'))
        rec['prev_fullName'] = prev   # None: the row was unmapped
        if beh:
            rec['behavior_file'] = beh   # file holding the logic (reprint classes only extend it)
    # Official text (tools/compare_official.py) overrides name-based guesses:
    # "use X" maps the row to that Twinleaf printing; otherwise Twinleaf has
    # no class with this printing's text, so the row stays unmapped.
    om = official.get('%s %s' % (r['set'], r['number']))
    if om and not (m and m['match'] in ('exact', 'exact-unregistered')):
        if om['verdict'].startswith('use '):
            fn = om['verdict'][4:]
            c = dump_by_full[fn]
            if rec.get('fullName') != fn:
                rec['prev_fullName'] = rec.get('fullName')
            rec.pop('error', None)
            rec.update(fullName=fn, cls=c['$class'], methods=c['$methods'], twinleaf_set=str(c['set']), twinleaf_number=str(c['setNumber']))
            has_logic = 'reduceEffect' in c['$methods'] or any(x in c['$methods'] for x in ('canPlay', 'canUseFromHandToBench'))
            rec.setdefault('tier', 'data' if not has_logic else 'custom')
            rec['official'] = 'same text as ' + fn
        else:
            if rec.get('fullName'):
                rec['prev_fullName'] = rec['fullName']
            for k in ('fullName', 'cls', 'methods', 'reduce_lines', 'effect_fns', 'tier', 'behavior_file'):
                rec.pop(k, None)
            rec['official'] = 'Twinleaf has no class with this printing\'s text (%s)' % om['verdict']
    rec['print_match'] = m['match'] if m else None
    rec['print_note'] = print_note(m, rec, prev, remapped)
    out.append(rec)
json.dump(out, open(os.path.join(ROOT, 'data/pool.json'), 'w'), indent=1)
print(collections.Counter(r.get('tier', r.get('error', 'absent')) for r in out))
print('remapped to exact printings:', sum(1 for r in out if 'prev_fullName' in r))
