"""Rebuild corpus traces after an oracle fix (PLAN.md 8.1 step 5).

Replays every trace through Rust (`diff`, invariants on), regenerates each
diverged trace with the current oracle from its own header (seed, decks,
policies, scenario: `cli.js regen`), and replays the regenerated traces. A
trace that still diverges after regeneration is a real parity bug (or an
invariant violation both engines share) and is listed for fixing.

usage: rebuild_corpus.py [dir ...] [--jobs J] [--dry-run]
       (default dirs: corpus/t1 corpus/meta1 corpus/meta2 corpus/cards/*/)

Writes porting/rebuild-<timestamp>.txt (local) with the regenerated and the
remaining traces.
"""
import argparse, glob, os, re, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = os.environ.get('PTCG_ORACLE') or os.path.join(ROOT, 'twinleaf/ptcg-server')
_bins = [os.path.join(ROOT, 'engine/target', p, 'diff') for p in ('release', 'iter')]
DIFF = os.environ.get('PTCG_DIFF') or max((b for b in _bins if os.path.exists(b)), key=os.path.getmtime)


def replay(paths):
    """First divergence per trace: {path: (step, what, detail)}."""
    out = {}
    p = subprocess.run([DIFF, *paths, '--quiet'], capture_output=True, text=True)
    for m in re.finditer(r'DIVERGED (\S+) at step (-?\d+): (\S+)\n  ([^\n]*)', p.stdout):
        out[os.path.abspath(m.group(1))] = (int(m.group(2)), m.group(3), m.group(4))
    return out


def regen(files):
    p = subprocess.run(['node', '--max-old-space-size=3072', 'output/oracle/cli.js', 'regen', *files],
                       cwd=ORACLE, capture_output=True, text=True)
    return p.stdout


def chunks(xs, n):
    for i in range(0, len(xs), n):
        yield xs[i:i + n]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('dirs', nargs='*')
    ap.add_argument('--jobs', type=int, default=max(1, (os.cpu_count() or 4) - 1))
    ap.add_argument('--dry-run', action='store_true')
    a = ap.parse_args()
    dirs = a.dirs or [os.path.join(ROOT, d) for d in ('corpus/t1', 'corpus/meta1', 'corpus/meta2')] + sorted(glob.glob(os.path.join(ROOT, 'corpus/cards/*/')))
    t0 = time.time()
    with ThreadPoolExecutor(a.jobs) as ex:
        found = {}
        for r in ex.map(lambda d: replay([d]), dirs):
            found.update(r)
    print('%d diverged traces in %d dirs (%.0fs)' % (len(found), len({os.path.dirname(f) for f in found}), time.time() - t0), flush=True)
    if a.dry_run or not found:
        return
    files = sorted(found)
    with ThreadPoolExecutor(a.jobs) as ex:
        logs = list(ex.map(regen, chunks(files, 25)))
    bad = [l for l in ''.join(logs).split('\n') if 'crashed' in l or 'status=error' in l or 'status=stuck' in l]
    with ThreadPoolExecutor(a.jobs) as ex:
        left = {}
        for r in ex.map(replay, chunks(files, 200)):
            left.update(r)
    stamp = time.strftime('%Y%m%d-%H%M')
    rep = os.path.join(ROOT, 'porting', 'rebuild-%s.txt' % stamp)
    with open(rep, 'w') as f:
        f.write('# rebuild %s: %d regenerated, %d still diverge, %d oracle error/stuck/crashed\n' % (stamp, len(files), len(left), len(bad)))
        for k, (s, w, d) in sorted(left.items()):
            f.write('DIVERGED %s at step %d: %s | %s\n' % (os.path.relpath(k, ROOT), s, w, d))
        for l in bad:
            f.write('ORACLE %s\n' % l)
    print('regenerated %d; still diverged %d; oracle failures %d -> %s (%.0fs)' % (len(files), len(left), len(bad), rep, time.time() - t0))
    sys.exit(1 if left else 0)


if __name__ == '__main__':
    main()
