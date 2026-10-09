"""Compare the diverged games in a `diff --quiet` output with tools/golden-expected.txt.

usage: golden_check.py <diff output file> [--expected FILE]

Exit status 0 only if the set of diverged games equals the expected set (and the output ends with
diff's summary line, so a crashed run is not read as clean). Update the expected file in the same
commit as a rules fix, with a `#` comment naming the attribution of each game.
"""
import argparse, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('output')
    ap.add_argument('--expected', default=os.path.join(HERE, 'golden-expected.txt'))
    a = ap.parse_args()
    text = open(a.output, errors='replace').read()
    got = set(re.findall(r'^DIVERGED\s+(?:\S*/)?(g\d+)\.json', text, re.M))
    exp = set()
    for line in open(a.expected):
        line = line.split('#')[0].strip()
        if line:
            exp.add(line)
    bad = 0
    m = re.search(r'^(\d+) traces: (\d+) pass .*?(\d+) diverged, (\d+) unsupported', text, re.M)
    if not m:
        print('no summary line in %s: diff did not finish' % a.output)
        bad += 1
    else:
        print(m.group(0))
        if int(m.group(4)):
            print('%s unsupported traces' % m.group(4))
            bad += 1
        if int(m.group(3)) != len(got):
            print('summary says %s diverged, %d parsed' % (m.group(3), len(got)))
            bad += 1
    for l in re.findall(r'^EXPECT FAILED.*$', text, re.M):
        print(l)
        bad += 1
    for g in sorted(got - exp):
        print('new divergence: %s' % g)
        bad += 1
    for g in sorted(exp - got):
        print('expected divergence missing: %s' % g)
        bad += 1
    if bad:
        sys.exit(1)
    print('golden: %d expected divergences, no other' % len(exp))


if __name__ == '__main__':
    main()
