"""Instructions per decision of the bench, counted by cachegrind (deterministic: no timer, no machine noise).

usage: instructions.py BENCH SPEC [--games N] [--base M] [--mode plain|host]... [--json OUT]

Runs `valgrind --tool=cachegrind` over `BENCH SPEC --games N --threads 1` and over the same with `--games M` (default
200 and 20), per mode (`plain`: the bench's random play; `host`: `bench --host`, the host calls). Both runs play the
same warm-up and their games are a prefix of each other's, so the difference of the two counts is exactly the
instructions of games M..N-1, and the difference of the two decision counts their decisions: the quotient is the
instructions per decision without the process start, the card tables, the spec load or the warm-up. (`--host` times
every call: its count includes the timer reads, about the same per decision in every commit.)

Prints one line per mode; appends a table to $GITHUB_STEP_SUMMARY when set; writes the numbers as JSON with --json
(the `instructions` artifact of the `rust` workflow, which tools/ci.py prints with the run's result).
Needs valgrind (Linux); CI's `instructions` job runs it.
"""
import argparse, json, os, re, subprocess, sys


def count(bench, spec, games, mode):
    cmd = ['valgrind', '--tool=cachegrind', '--cache-sim=no', '--cachegrind-out-file=/dev/null',
           bench, spec, '--games', str(games), '--threads', '1'] + (['--host'] if mode == 'host' else [])
    r = subprocess.run(cmd, text=True, capture_output=True)
    if r.returncode != 0:
        sys.exit('%s failed (%d):\n%s%s' % (' '.join(cmd), r.returncode, r.stdout, r.stderr[-3000:]))
    ir = re.search(r'I\s+refs:\s+([\d,]+)', r.stderr)
    dec = re.search(r'(\d+) decisions', r.stdout)
    if not ir or not dec:
        sys.exit('no instruction or decision count in the output of %s:\n%s%s' % (' '.join(cmd), r.stdout, r.stderr[-3000:]))
    return int(ir.group(1).replace(',', '')), int(dec.group(1))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('bench')
    ap.add_argument('spec')
    ap.add_argument('--games', type=int, default=200)
    ap.add_argument('--base', type=int, default=20)
    ap.add_argument('--mode', action='append', choices=['plain', 'host'])
    ap.add_argument('--json')
    a = ap.parse_args()
    if a.base >= a.games:
        sys.exit('--base must be below --games')
    out = {}
    for mode in a.mode or ['plain']:
        ir_n, d_n = count(a.bench, a.spec, a.games, mode)
        ir_m, d_m = count(a.bench, a.spec, a.base, mode)
        per = (ir_n - ir_m) / max(1, d_n - d_m)
        out[mode] = {'per_decision': round(per), 'instructions': ir_n - ir_m, 'decisions': d_n - d_m,
                     'games': [a.base, a.games]}
        print('%s: %.0f instructions per decision (games %d..%d: %d decisions, %d instructions)'
              % (mode, per, a.base, a.games - 1, d_n - d_m, ir_n - ir_m))
    summary = os.environ.get('GITHUB_STEP_SUMMARY')
    if summary:
        with open(summary, 'a') as f:
            f.write('### Instructions per decision (cachegrind, bench games %d..%d)\n\n' % (a.base, a.games - 1))
            f.write('| mode | instructions / decision | decisions |\n|---|---|---|\n')
            for mode, v in out.items():
                f.write('| %s | %s | %d |\n' % (mode, format(v['per_decision'], ','), v['decisions']))
    if a.json:
        with open(a.json, 'w') as f:
            json.dump(out, f, indent=1)


if __name__ == '__main__':
    main()
