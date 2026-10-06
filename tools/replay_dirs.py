"""Replay every corpus directory under a root through $PTCG_DIFF in parallel (for running on a remote machine).

usage: replay_dirs.py ROOT OUT [--procs N]
Writes OUT/replay.txt: the DIVERGED blocks and per-directory summary lines.
"""
import argparse, glob, os, re, subprocess
from concurrent.futures import ThreadPoolExecutor

ap = argparse.ArgumentParser()
ap.add_argument('root')
ap.add_argument('out')
ap.add_argument('--procs', type=int, default=os.cpu_count() or 8)
a = ap.parse_args()
root = os.path.expanduser(a.root)
dirs = [os.path.join(root, d) for d in ('t1', 'meta1', 'meta2') if os.path.isdir(os.path.join(root, d))]
dirs += sorted(glob.glob(os.path.join(root, 'cards', '*/')))


def run(d):
    r = subprocess.run([os.environ['PTCG_DIFF'], d, '--quiet'], capture_output=True, text=True)
    return r.stdout + r.stderr


os.makedirs(a.out, exist_ok=True)
tot = {'traces': 0, 'pass': 0, 'diverged': 0, 'unsupported': 0}
with open(os.path.join(a.out, 'replay.txt'), 'w') as f, ThreadPoolExecutor(a.procs) as ex:
    for out in ex.map(run, dirs):
        f.write(out.replace(root.rstrip('/') + '/', 'corpus/'))
        m = re.search(r'(\d+) traces: (\d+) pass .*?(\d+) diverged, (\d+) unsupported', out)
        if m:
            for k, v in zip(('traces', 'pass', 'diverged', 'unsupported'), map(int, m.groups())):
                tot[k] += v
    f.write('TOTAL %s\n' % tot)
print('TOTAL', tot)
