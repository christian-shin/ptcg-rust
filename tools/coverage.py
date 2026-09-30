"""Per-card branch coverage from oracle runs (PLAN.md 4.5).

usage: coverage.py <cov_dir> <twinleaf_file.ts> [...] [--min 10]

`cov_dir` holds V8 coverage snapshots written by `cli.js corpus` when run
with NODE_V8_COVERAGE (one snapshot per game). For every block of each card's
compiled JavaScript, counts the number of distinct games in which it ran and
lists blocks hit in fewer than `--min` games, with a source excerpt.
Blocks that never ran are reachable-or-not candidates for the exemption list.
"""
import argparse, collections, glob, json, os, re

ORACLE = os.environ.get('PTCG_ORACLE', '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server')


def snapshots(cov_dir):
    by_pid = collections.defaultdict(list)
    for f in glob.glob(os.path.join(cov_dir, 'coverage-*.json')):
        m = re.match(r'coverage-(\d+)-(\d+)-(\d+)\.json', os.path.basename(f))
        if m:
            by_pid[m.group(1)].append((int(m.group(2)), int(m.group(3)), f))
    for pid in by_pid:
        by_pid[pid].sort()
    return by_pid


def block_counts(snapshot_file, urls):
    data = json.load(open(snapshot_file))
    out = {}
    for script in data.get('result', []):
        url = script.get('url', '')
        if url not in urls:
            continue
        for fn in script['functions']:
            for r in fn['ranges']:
                out[(url, fn['functionName'], r['startOffset'], r['endOffset'])] = r['count']
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('cov_dir')
    ap.add_argument('files', nargs='+')
    ap.add_argument('--min', type=int, default=10)
    args = ap.parse_args()
    js = {}
    for f in args.files:
        path = os.path.join(ORACLE, 'output', f[:-3] + '.js') if f.endswith('.ts') else f
        js['file://' + os.path.abspath(path)] = f
    hits = collections.Counter()
    seen = set()
    games = 0
    for pid, snaps in snapshots(args.cov_dir).items():
        prev = {}
        for _, _, f in snaps:
            cur = block_counts(f, js)
            games += 1
            for k, c in cur.items():
                seen.add(k)
                if c - prev.get(k, 0) > 0:
                    hits[k] += 1
            prev = cur
    print('%d game snapshots' % games)
    for url, name in js.items():
        src = open(url[len('file://'):]).read() if os.path.exists(url[len('file://'):]) else ''
        blocks = sorted(k for k in seen if k[0] == url)
        if not blocks:
            print('%s: not loaded / never ran' % name)
            continue
        low = [k for k in blocks if hits[k] < args.min]
        print('%s: %d blocks, %d hit in >= %d games' % (name, len(blocks), len(blocks) - len(low), args.min))
        for (_, fn, s, e) in low:
            if e - s > 4000:
                continue  # whole-module / class wrapper ranges
            line = src.count('\n', 0, s) + 1
            snippet = ' '.join(src[s:e].split())[:110]
            print('  %3d games  %s:%d  %s' % (hits[(url, fn, s, e)], fn or '<anon>', line, snippet))


if __name__ == '__main__':
    main()
