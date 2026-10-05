"""Tier 4 gate ledger: green games per code version across GitHub Actions runs.

Every tier4.yml shard logs `tier4-code main=<sha> oracle=<sha>` and a
`[tier4 seed S] GREEN|RED: N games, ... K failures` line. This tool reads the
completed runs, caches their per-shard results in porting/tier4-ledger.json
(local), and reports for each (main, oracle) pair how many fresh games were
played and how many failed. The gate (PLAN.md 8.2) is 100,000 games with 0
failures on the current pair.

usage: tier4_ledger.py [--limit N] [--download] [--gate 100000]
  --download   fetch the failing traces of red runs into corpus/tier4/ci/<run>/
"""
import argparse, json, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPO = os.environ.get('PTCG_REPO', 'christian-shin/ptcg-rust')
CACHE = os.path.join(ROOT, 'porting', 'tier4-ledger.json')
CODE = re.compile(r'tier4-code main=([0-9a-f]+) oracle=([0-9a-f]+)')
RESULT = re.compile(r'\[tier4 seed (\d+)\] (GREEN|RED): (\d+) games, (\d+) steps, (\d+) failures')


def gh(*args):
    return subprocess.run(['gh', *args], capture_output=True, text=True).stdout


def run_results(run_id):
    """Per-shard (main, oracle, seed, games, failures) of one completed run."""
    log = gh('run', 'view', str(run_id), '--repo', REPO, '--log')
    code, out = {}, []
    for line in log.split('\n'):
        job = line.split('\t', 1)[0]
        m = CODE.search(line)
        if m:
            code[job] = m.groups()
        m = RESULT.search(line)
        if m and job in code:
            out.append({'main': code[job][0], 'oracle': code[job][1], 'seed': int(m.group(1)),
                        'games': int(m.group(3)), 'failures': int(m.group(5))})
    # A shard that crashed or timed out has no result line: it counts as red.
    missing = [j for j in code if not any(RESULT.search(l) for l in log.split('\n') if l.startswith(j + '\t'))]
    for j in missing:
        out.append({'main': code[j][0], 'oracle': code[j][1], 'seed': -1, 'games': 0, 'failures': 1, 'note': 'no result: ' + j})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--limit', type=int, default=200)
    ap.add_argument('--download', action='store_true')
    ap.add_argument('--gate', type=int, default=100000)
    a = ap.parse_args()
    cache = json.load(open(CACHE)) if os.path.exists(CACHE) else {}
    runs = json.loads(gh('run', 'list', '--repo', REPO, '--workflow', 'tier4.yml', '--limit', str(a.limit),
                         '--json', 'databaseId,status,conclusion,createdAt,event'))
    for r in runs:
        rid = str(r['databaseId'])
        if r['status'] != 'completed' or rid in cache or r['conclusion'] == 'cancelled':
            continue
        cache[rid] = {'created': r['createdAt'], 'event': r['event'], 'conclusion': r['conclusion'], 'shards': run_results(rid)}
    json.dump(cache, open(CACHE, 'w'), indent=1)

    main_sha = subprocess.run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], capture_output=True, text=True).stdout.strip()
    oracle_sha = subprocess.run(['git', '-C', os.path.join(ROOT, 'twinleaf'), 'rev-parse', 'oracle'], capture_output=True, text=True).stdout.strip()
    pairs = {}
    for rid, r in cache.items():
        for s in r['shards']:
            k = (s['main'], s['oracle'])
            p = pairs.setdefault(k, {'games': 0, 'failures': 0, 'red_runs': set(), 'runs': set()})
            p['games'] += s['games']
            p['failures'] += s['failures']
            p['runs'].add(rid)
            if s['failures']:
                p['red_runs'].add(rid)
    for (m, o), p in sorted(pairs.items(), key=lambda kv: -kv[1]['games'])[:8]:
        cur = ' <- current' if m == main_sha and o.startswith(oracle_sha[:9]) else ''
        print('main %s oracle %s: %d games in %d runs, %d failures%s' % (m[:9], o[:9], p['games'], len(p['runs']), p['failures'], cur))
    cur = next((p for (m, o), p in pairs.items() if m == main_sha and o.startswith(oracle_sha[:9])), None)
    if cur is None:
        print('current code (main %s, oracle %s): no completed runs yet' % (main_sha[:9], oracle_sha[:9]))
        sys.exit(1)
    green = cur['failures'] == 0 and cur['games'] >= a.gate
    print('GATE %s: %d / %d games on the current code, %d failures' % ('GREEN' if green else 'open', cur['games'], a.gate, cur['failures']))
    if a.download:
        for rid in sorted(cur['red_runs']):
            d = os.path.join(ROOT, 'corpus/tier4/ci', rid)
            if not os.path.isdir(d):
                os.makedirs(d)
                gh('run', 'download', rid, '--repo', REPO, '--dir', d)
                print('downloaded failures of run %s -> %s' % (rid, os.path.relpath(d, ROOT)))
    sys.exit(0 if green else 1)


if __name__ == '__main__':
    main()
