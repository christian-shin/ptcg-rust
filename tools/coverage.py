"""Per-card branch coverage from oracle runs (PLAN.md 4.5).

usage: coverage.py <cov_dir[,cov_dir...]> <twinleaf_file.ts> [...] [--min N]

`cov_dir` holds V8 coverage snapshots written by `cli.js corpus` under
NODE_V8_COVERAGE, one per game. `v8.takeCoverage()` resets counts at each
snapshot, and ranges coalesce differently per snapshot, so the file is cut at
the union of all range boundaries and each segment's innermost-range count is
read per game. Lists segments that ran in fewer than N games (default 10).
"""
import collections, glob, json, os, sys

ORACLE = os.environ.get('PTCG_ORACLE', '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server')
cov_dir = sys.argv[1]
args = sys.argv[2:]
mn = 10
files = []
i = 0
while i < len(args):
    if args[i] == '--min':
        mn = int(args[i + 1]); i += 2; continue
    if args[i].startswith('--min='):
        mn = int(args[i][6:]); i += 1; continue
    files.append(args[i]); i += 1
urls = {'file://' + os.path.join(ORACLE, 'output', f[:-3] + '.js'): f for f in files}
snaps = sorted(f for d in cov_dir.split(',') for f in glob.glob(os.path.join(d, 'coverage-*.json')))
per_url_bounds = collections.defaultdict(set)
games_ranges = []
for f in snaps:
    d = json.load(open(f))
    g = {}
    for s in d.get('result', []):
        if s['url'] in urls:
            rs = []
            for fn in s['functions']:
                for r in fn['ranges']:
                    rs.append((r['startOffset'], r['endOffset'], r['count']))
                    per_url_bounds[s['url']].add(r['startOffset'])
                    per_url_bounds[s['url']].add(r['endOffset'])
            g[s['url']] = rs
    games_ranges.append(g)


def count_at(rs, off):
    best = None
    for (s, e, c) in rs:
        if s <= off < e and (best is None or e - s < best[1] - best[0]):
            best = (s, e, c)
    return None if best is None else best[2]


for url, name in urls.items():
    src = open(url[7:]).read()
    bounds = sorted(per_url_bounds[url])
    segs = [(bounds[i], bounds[i + 1]) for i in range(len(bounds) - 1)]
    low = []
    for (s, e) in segs:
        if not src[s:e].strip() or e - s > 4000:
            continue
        hits = 0
        seen = False
        for g in games_ranges:
            rs = g.get(url)
            if rs is None:
                continue
            c = count_at(rs, s)
            if c is None:
                continue
            seen = True
            if c > 0:
                hits += 1
        if seen and hits < mn:
            low.append((s, e, hits))
    print('%s: %d segments, %d below %d games' % (name, len(segs), len(low), mn))
    for (s, e, h) in low:
        line = src.count('\n', 0, s) + 1
        print('  %3d games  L%d  %s' % (h, line, ' '.join(src[s:e].split())[:120]))
