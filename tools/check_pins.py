"""Check port pins: a port pinned to some printings (`class: "Class@SET"`)
must only be pinned when Twinleaf defines that class name in more than one
file. When the class is defined once, every printing whose behavior is that
class is the same card logic (reprints extend it), so the port must be
unpinned or it leaves the other printings unported.

usage: check_pins.py [impls_dir]    (default engine/src/cards/impls)
Exit status 1 if an unnecessary pin leaves a printing unbound.
"""
import collections, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
impls = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, 'engine/src/cards/impls')
sets = os.path.join(ROOT, 'twinleaf/ptcg-server/src/sets')
if not os.path.isdir(sets):   # worktrees have no twinleaf/ checkout
    sets = '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server/src/sets'
count = collections.Counter()
for d, _, fs in os.walk(sets):
    for f in fs:
        if f.endswith('.ts'):
            count.update(re.findall(r'export class (\w+)', open(os.path.join(d, f), encoding='utf-8').read()))
gen = open(os.path.join(ROOT, 'engine/src/gen/cards.rs'), encoding='utf-8').read()
by_class = collections.defaultdict(list)
for m in re.finditer(r'full_name: "((?:[^"\\]|\\.)*)", name: "(?:[^"\\]|\\.)*", set: "([^"]*)".*?behavior: "([^"]*)"', gen):
    by_class[m.group(3)].append((m.group(1), m.group(2)))
bad = 0
for f in sorted(os.listdir(impls)):
    if not f.endswith('.rs'):
        continue
    for cl in re.findall(r'class: "([^"]*)"', open(os.path.join(impls, f), encoding='utf-8').read()):
        cls, _, pins = cl.partition('@')
        if not pins or count[cls] > 1:
            continue
        alts = pins.split('|')
        unbound = [fn for fn, st in by_class[cls] if fn not in alts and st not in alts]
        if unbound:
            bad += 1
            print('%s: %s is defined once in Twinleaf, so the pin @%s is unnecessary and leaves %s unbound; '
                  'use class: "%s"' % (f, cls, pins, ', '.join(unbound), cls))
print('pins ok' if not bad else '%d unnecessary pin(s)' % bad)
sys.exit(1 if bad else 0)
