"""Run every scenario (or those given) through check_cards.py in parallel, for vbox exec.

usage: run_scenarios.py OUT [--games N] [--procs P] [files...]
Target = the first card of the scenario's first deck. Writes OUT/scenarios.txt
(one line per scenario: exit code, file, result line, warnings).
"""
import argparse, glob, json, os, re, subprocess
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ap = argparse.ArgumentParser()
ap.add_argument('out')
ap.add_argument('--games', type=int, default=4)
ap.add_argument('--procs', type=int, default=os.cpu_count() or 8)
ap.add_argument('files', nargs='*')
a = ap.parse_args()
files = a.files or sorted(glob.glob(os.path.join(ROOT, 'scenarios', '*.json')))


def target(sc):
    d = sc['decks'][0][0]
    m = re.match(r'^\d+ (.*)$', d)
    return m.group(1) if m else d


def run(f):
    sc = json.load(open(f))
    slug = os.path.basename(f)[:-5]
    r = subprocess.run(['python3', 'tools/check_cards.py', target(sc), '--scenario', f, '--games', str(a.games),
                        '--jobs', '1', '--tag', 'sc-' + slug, '--out', os.path.join(a.out, 'corpus', slug)],
                       cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    keep = [l for l in out.split('\n') if ' traces: ' in l or 'WARNING' in l or 'ORACLE' in l or 'rror' in l]
    return '%d %s %s' % (r.returncode, os.path.relpath(f, ROOT), ' | '.join(keep[-4:]))


os.makedirs(a.out, exist_ok=True)
bad = 0
with open(os.path.join(a.out, 'scenarios.txt'), 'w') as fh, ThreadPoolExecutor(a.procs) as ex:
    for line in ex.map(run, files):
        fh.write(line + '\n')
        if not line.startswith('0 ') or ' 0 diverged' not in line or 'WARNING' in line:
            bad += 1
            print(line, flush=True)
print('scenarios: %d run, %d not clean' % (len(files), bad))
