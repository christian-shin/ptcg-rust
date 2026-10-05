"""Regenerate one shard of corpus traces from their headers on GitHub Actions
(.github/workflows/regen.yml), then replay them through Rust.

usage: regen_shard.py HEADERS.json.gz SHARD SHARDS OUT [--jobs J]

HEADERS maps a corpus-relative trace path to its header (tools: pack the
headers of the diverged traces locally). Each shard writes header-only stubs
under OUT/<path>, runs `cli.js regen` on them (which re-plays the game from
seed, decks, policies and scenario with the current oracle), replays the
result with `diff`, and writes OUT/summary-<SHARD>.txt.
"""
import argparse, gzip, json, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ap = argparse.ArgumentParser()
ap.add_argument('headers')
ap.add_argument('shard', type=int)
ap.add_argument('shards', type=int)
ap.add_argument('out')
ap.add_argument('--jobs', type=int, default=4)
a = ap.parse_args()
ORACLE = os.environ['PTCG_ORACLE']
DIFF = os.environ['PTCG_DIFF']

with gzip.open(a.headers, 'rt') as g:
    headers = json.load(g)
mine = sorted(headers)[a.shard::a.shards]
files = []
for rel in mine:
    p = os.path.abspath(os.path.join(a.out, rel))
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, 'w') as f:
        json.dump({'header': headers[rel]}, f)
    files.append(p)
print('shard %d/%d: %d traces' % (a.shard, a.shards, len(files)), flush=True)


def regen(chunk):
    r = subprocess.run(['node', '--max-old-space-size=3072', 'output/oracle/cli.js', 'regen', *chunk],
                       cwd=ORACLE, capture_output=True, text=True)
    return r.stdout + r.stderr


chunks = [files[i:i + 10] for i in range(0, len(files), 10)]
with ThreadPoolExecutor(a.jobs) as ex:
    logs = []
    for i, l in enumerate(ex.map(regen, chunks)):
        logs.append(l)
        if i % 10 == 0:
            print('regen chunk %d/%d' % (i, len(chunks)), flush=True)
log = ''.join(logs)
bad = [l for l in log.split('\n') if 'crashed' in l or 'status=error' in l or 'status=stuck' in l]

left = []
summary = []
for i in range(0, len(files), 200):
    r = subprocess.run([DIFF, *files[i:i + 200], '--quiet'], capture_output=True, text=True)
    left += re.findall(r'DIVERGED (\S+) at step (-?\d+): (\S+)\n  ([^\n]*)', r.stdout)
    summary += [l for l in r.stdout.split('\n') if ' traces: ' in l]
with open(os.path.join(a.out, 'summary-%d.txt' % a.shard), 'w') as f:
    f.write('# shard %d: %d regenerated, %d still diverge, %d oracle error/stuck/crashed\n' % (a.shard, len(files), len(left), len(bad)))
    for p, s, w, d in left:
        f.write('DIVERGED %s at step %s: %s | %s\n' % (os.path.relpath(p, a.out), s, w, d[:300]))
    for l in bad:
        f.write('ORACLE %s\n' % l.replace(os.path.abspath(a.out) + '/', ''))
    for l in summary:
        f.write('# %s\n' % l)
print(open(os.path.join(a.out, 'summary-%d.txt' % a.shard)).read()[:5000])
sys.exit(1 if left else 0)
