"""Rename every card behavior class key to an official-name key (batch 9, C14a).

A card file (engine/src/cards/impls/*.rs) declares `class: "<key>"`, data/cards.json gives each card a `behavior`
naming the class that carries its logic, and engine/src/cards/mod.rs matched the two by `class_matches` (a key
optionally qualified `@SET`, `@Full Name` or `@A|B`). The old keys were Twinleaf class names. This script derives
one official key per implementation from the official English card name:

  - the key is the card name in identifier form ("Neutralization Zone" -> NeutralizationZone, "Meowth ex" ->
    MeowthEx, "Hop's Phantump" -> HopsPhantump, "Poké Pad" -> PokePad);
  - when two different implementations carry the same card name (different text), every one of them gets the set
    code of the printing whose text it implements (CrustleBLK); when a set code is not enough, the card number too;
  - an implementation shared by reprints keeps one key (the first printing its old qualifier names, else the card
    that first resolves to it);
  - a behavior class no file implements yet (a card whose behavior is not ported) gets a key by the same rule, so
    every card's `behavior` is the exact key of its class and registry matching is plain equality.

The old per-card `class` field of data/cards.json (read by nothing) is dropped.

usage (from the repo root; standard library only):
  python3 -I tools/rename_classes.py table      print the full old -> new table, refuse on any collision
  python3 -I tools/rename_classes.py frozen     write engine/src/cards/impl_table.txt (card -> key, by the OLD matching)
  python3 -I tools/rename_classes.py apply      rewrite data/cards.json, the card files, regenerate engine/src/gen
Run `frozen` before `apply`; `apply` refuses on a tree that is already renamed.
"""
import collections, glob, json, os, re, subprocess, sys, unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IMPLS = os.path.join(ROOT, 'engine/src/cards/impls')
CARDS = os.path.join(ROOT, 'data/cards.json')
FROZEN = os.path.join(ROOT, 'engine/src/cards/impl_table.txt')
CLASS_RE = re.compile(r'(\bclass: ")([^"]*)(")')


def load():
    cards = json.load(open(CARDS))
    files = {}  # stem -> old class (None if the file declares none)
    for f in sorted(glob.glob(IMPLS + '/*.rs')):
        m = CLASS_RE.findall(open(f).read())
        assert len(m) <= 1, f
        files[os.path.basename(f)[:-3]] = m[0][1] if m else None
    return cards, files


def class_matches(cls, key, c):
    """The old rule of engine/src/cards/mod.rs."""
    if '@' in cls:
        b, q = cls.split('@', 1)
        return b == c['behavior'] and any(x in (c['set'], key) for x in q.split('|'))
    return cls == c['behavior']


def words(name):
    s = unicodedata.normalize('NFKD', name)
    s = ''.join(ch for ch in s if not unicodedata.combining(ch))
    s = s.replace("'", '').replace('’', '').replace('´', '')
    return re.findall(r'[A-Za-z0-9]+', s)


def ident(name):
    return ''.join(w[0].upper() + w[1:] for w in words(name))


def groups(cards, files):
    """Group every card with a behavior by the implementation the OLD matching gives it (`stem`), or by its old
    behavior when no file matches it (`('-', behavior)`). Returns (group -> [card keys], card key -> group)."""
    by_group = collections.OrderedDict()
    of = {}
    for k, c in cards.items():
        if not c['behavior']:
            continue
        g = None
        for stem, cls in files.items():  # file order: the order build.rs registers them
            if cls is not None and class_matches(cls, k, c):
                g = stem
                break
        if g is None:
            g = ('-', c['behavior'])
        by_group.setdefault(g, []).append(k)
        of[k] = g
    return by_group, of


def derive(cards, files):
    by_group, of = groups(cards, files)
    problems = []
    info = {}  # group -> (name, set, number, old key)
    for g, ks in by_group.items():
        names = {cards[k]['name'] for k in ks}
        if len(names) != 1:
            problems.append('%s: one class, several card names %s' % (g, sorted(names)))
            continue
        old = files[g] if isinstance(g, str) else g[1]
        pick = ks[0]
        if isinstance(g, str) and '@' in old:
            for q in old.split('@', 1)[1].split('|'):  # the first printing the qualifier names
                hit = [k for k in ks if q in (cards[k]['set'], k)]
                if hit:
                    pick = hit[0]
                    break
        c = cards[pick]
        info[g] = (c['name'], c['set'], c['setNumber'], old)
    base = collections.defaultdict(list)
    for g, (name, st, num, old) in info.items():
        base[ident(name)].append(g)
    key = {}
    for b, gs in base.items():
        if len(gs) == 1:
            key[gs[0]] = b
            continue
        with_set = collections.defaultdict(list)
        for g in gs:
            with_set[b + ident(info[g][1])].append(g)
        for k2, g2 in with_set.items():
            if len(g2) == 1:
                key[g2[0]] = k2
            else:
                for g in g2:
                    key[g] = k2 + ident(info[g][2])
    seen = collections.defaultdict(list)
    for g, k in key.items():
        seen[k].append(g)
        if not re.fullmatch(r'[A-Z][A-Za-z0-9]*', k):
            problems.append('%s: key %r is not an identifier' % (g, k))
    for k, gs in seen.items():
        if len(gs) > 1:
            problems.append('collision on %s: %s' % (k, gs))
    return by_group, of, info, key, problems


def table():
    cards, files = load()
    by_group, of, info, key, problems = derive(cards, files)
    for g in by_group:
        if g in info:
            print('%-44s -> %-34s %s' % (info[g][3], key[g], '(file %s)' % g if isinstance(g, str) else '(no file)'))
    unused = [s for s, cl in files.items() if cl is None or s not in by_group]
    if unused:
        problems.append('files that no card resolves to (or without a class): %s' % unused)
    for p in problems:
        print('REFUSE:', p)
    n_files = sum(1 for g in by_group if isinstance(g, str))
    print('%d groups (%d implemented, %d without a file), %d cards with a behavior' % (len(by_group), n_files, len(by_group) - n_files, len(of)))
    return cards, files, by_group, of, key, problems


def frozen():
    cards, files, by_group, of, key, problems = table()
    if problems:
        sys.exit(1)
    with open(FROZEN, 'w') as f:
        f.write('# card (international key) -> the class key of the implementation it resolved to when the keys were\n'
                '# renamed (tools/rename_classes.py frozen, made with the old matching); "-" = behavior not ported.\n')
        for k, g in of.items():
            f.write('%s\t%s\n' % (k, key[g] if isinstance(g, str) else '-'))
    print('wrote', FROZEN)


def apply():
    cards, files, by_group, of, key, problems = table()
    if problems:
        sys.exit(1)
    if not os.path.exists(FROZEN):
        sys.exit('run `frozen` first')
    # data/cards.json: text-level, one card per line, so the layout is unchanged
    out = []
    for line in open(CARDS).read().split('\n'):
        m = re.match(r'^("(?:[^"\\]|\\.)*"):\{(.*)$', line)
        if not m:
            out.append(line)
            continue
        k = json.loads(m.group(1))
        rest = m.group(2)
        rest, n = re.subn(r'"class":"[^"]*",', '', rest, count=1)
        assert n == 1, k
        new = key[of[k]] if k in of else ''
        rest, n = re.subn(r'"behavior":"[^"]*"', '"behavior":%s' % json.dumps(new), rest, count=1)
        assert n == 1, k
        out.append('%s:{%s' % (m.group(1), rest))
    open(CARDS, 'w').write('\n'.join(out))
    # the card files
    for stem, cls in files.items():
        p = os.path.join(IMPLS, stem + '.rs')
        t = open(p).read()
        new = key[stem]
        t, n = CLASS_RE.subn(lambda m: m.group(1) + new + m.group(3), t)
        assert n == 1, p
        open(p, 'w').write(t)
    subprocess.check_call([sys.executable, '-I', os.path.join(ROOT, 'tools/gen_carddb.py')])


if __name__ == '__main__':
    cmd = sys.argv[1] if len(sys.argv) > 1 else ''
    if cmd == 'table':
        sys.exit(1 if table()[-1] else 0)
    elif cmd == 'frozen':
        frozen()
    elif cmd == 'apply':
        apply()
    else:
        sys.exit(__doc__)
