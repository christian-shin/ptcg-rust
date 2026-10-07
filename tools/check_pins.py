"""Check port pins: a port pinned to some printings (`class: "Class@SET"`)
must only be pinned when Twinleaf defines that class name in more than one
file. When the class is defined once, every printing whose behavior is that
class is the same card logic (reprints extend it), so the port must be
unpinned or it leaves the other printings unported.

Reverse check: an UNPINNED port whose class name Twinleaf defines in more than
one file must not be bound by cards whose behavior comes from different files
(different cards that share a class name); those need pinned ports.

usage: check_pins.py [impls_dir]    (default engine/src/cards/impls)
Exit status 1 if an unnecessary pin leaves a printing unbound.
"""
import collections, json, os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import names

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
impls = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, 'engine/src/cards/impls')
sets = os.path.join(ROOT, 'twinleaf/ptcg-server/src/sets')
if not os.path.isdir(sets):   # worktrees have no twinleaf/ checkout
    sets = '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server/src/sets'
count = collections.Counter()
src_root = os.path.dirname(sets)          # .../src
defs = {}                                  # fullName -> (file relative to src, class)
extends = {}                               # (file, class) -> base class
for d, _, fs in os.walk(sets):
    for f in fs:
        if f.endswith('.ts'):
            path = os.path.join(d, f)
            text = open(path, encoding='utf-8').read()
            count.update(re.findall(r'export class (\w+)', text))
            rel = os.path.relpath(path, src_root)
            for m in re.finditer(r'export class (\w+)(?: extends (\w+))?', text):
                extends[(rel, m.group(1))] = m.group(2)
            for m in re.finditer(r"fullName(?:: string)?\s*=\s*'((?:[^'\\]|\\.)*)'", text):
                cm = re.findall(r'export class (\w+)', text[:m.start()])
                if cm:
                    defs.setdefault(m.group(1).replace("\\'", "'"), (rel, cm[-1]))
gen = open(os.path.join(ROOT, 'engine/src/gen/cards.rs'), encoding='utf-8').read()
by_class = collections.defaultdict(list)
for m in re.finditer(r'tl_full_name: "((?:[^"\\]|\\.)*)", tl_name: "(?:[^"\\]|\\.)*", tl_set: "([^"]*)".*?behavior: "([^"]*)"', gen):
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
                  'use class: "%s"' % (f, cls, pins, ', '.join(names.label(u) for u in unbound), cls))


def behavior_file(full_name, behavior):
    """Twinleaf file holding the logic class `behavior` for a support card."""
    if full_name not in defs:
        return None
    f, c = defs[full_name]
    for _ in range(10):
        if c == behavior:
            return f
        base = extends.get((f, c))
        if not base:
            return f
        if (f, base) in extends:          # base defined in the same file
            c = base
            continue
        text = open(os.path.join(src_root, f), encoding='utf-8').read()
        m = re.search(r'import\s*\{[^}]*\b%s\b[^}]*\}\s*from\s*\'([^\']+)\'' % base, text)
        if not m:
            return f
        nf = os.path.normpath(os.path.join(os.path.dirname(f), m.group(1))) + '.ts'
        if not os.path.exists(os.path.join(src_root, nf)):
            return f
        f, c = nf, base
    return f


# Reverse check: unpinned ports shared by different Twinleaf files.
file_of = {}                               # fullName -> behavior file
for row in json.load(open(os.path.join(ROOT, 'data/pool.json'), encoding='utf-8')):
    if row.get('fullName'):
        file_of[row['fullName']] = row.get('behavior_file') or row.get('twinleaf_file')
behavior_of = {fn: cl for cl, lst in by_class.items() for fn, _ in lst}
support = os.path.join(ROOT, 'data/support_cards.json')
if os.path.exists(support):
    for row in json.load(open(support, encoding='utf-8')):
        fn = row.get('fullName')
        if fn and fn not in file_of and fn in behavior_of:
            file_of[fn] = behavior_file(fn, behavior_of[fn])
unpinned = set()
for f in sorted(os.listdir(impls)):
    if f.endswith('.rs'):
        for cl in re.findall(r'class: "([^"]*)"', open(os.path.join(impls, f), encoding='utf-8').read()):
            if '@' not in cl:
                unpinned.add(cl)
for cls in sorted(unpinned):
    if count[cls] <= 1:
        continue
    by_file = collections.defaultdict(list)
    for fn, _ in by_class[cls]:
        if file_of.get(fn):
            by_file[file_of[fn]].append(fn)
    if len(by_file) >= 2:
        bad += 1
        print('unpinned port %s binds cards from different Twinleaf files (pin it per printing):' % cls)
        for bf, fns in sorted(by_file.items()):
            print('    %s: %s' % (bf, ', '.join(names.label(u) for u in fns)))
print('pins ok' if not bad else '%d pin problem(s)' % bad)
sys.exit(1 if bad else 0)
