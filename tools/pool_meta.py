"""Join card-pool.tsv with the Twinleaf card dump -> data/pool.json.

Each pool row gets the Twinleaf card record (fullName, class, methods) found
by (twinleaf_set, twinleaf_number), and a tier guess:
  data    - no reduceEffect (printed data only)
  short   - reduceEffect / effect bodies short enough to be prefab calls
  custom  - everything else
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
    out.append(rec)
json.dump(out, open(os.path.join(ROOT, 'data/pool.json'), 'w'), indent=1)
print(collections.Counter(r.get('tier', r.get('error', 'absent')) for r in out))
