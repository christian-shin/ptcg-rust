"""Verify cards in the Rust engine (Rust-only since the
oracle freeze of 2026-10-08).

usage: check_cards.py "Card A" ["Card B" ...]   (international key such as
                      "Growing Grass Energy POR 86", or "Name SET" when unique)
                      [--games N] [--seed S] [--out DIR] [--tag TAG] [--keep]
                      [--scenario JSON] [--no-scenarios] [--threads T]

1. Builds decks that contain every target card (4 copies, or 1 for ACE SPEC),
   filled with printed-data Pokémon and ported cards, plus basic energy that
   pays the targets' attack costs. Stage 1/2 targets need their
   pre-evolution among the targets or already ported/data cards.
2. Plays N games with `fuzz` (heur and random policies): engine errors, stuck
   prompts, turns without options, invariant violations and panics fail the
   check; each failing game's trace is written to the out dir (`diff` replays
   it). `--keep` keeps every game's trace.
3. Prints how often the targets acted (`scout`, same decks), so a card that
   never got to do anything is visible.
4. Runs every scenario that mentions a target with `scen` (unless
   --no-scenarios). With `--scenario JSON`, runs only that scenario, N games.

Output goes to corpus/cards/<tag>/ (default tag: first target, slugified).
Exit status 1 on any failure.
"""
import argparse, collections, glob, json, os, random, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _newest(*paths):
    have = [p for p in paths if os.path.exists(p)]
    return max(have, key=os.path.getmtime) if have else paths[0]


def _bin(name):
    return _newest(os.path.join(ROOT, 'engine/target/release', name), os.path.join(ROOT, 'engine/target/iter', name))


DIFF = os.environ.get('PTCG_DIFF') or _bin('diff')
FUZZ = _bin('fuzz')
SCEN = _bin('scen')
SCOUT = _bin('scout')

cards = json.load(open(os.path.join(ROOT, 'data/cards.json')))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
BASIC_ENERGY = {1: 'Grass Energy MEE 1', 2: 'Fire Energy MEE 2', 3: 'Water Energy MEE 3', 4: 'Lightning Energy MEE 4',
                5: 'Psychic Energy MEE 5', 6: 'Fighting Energy MEE 6', 7: 'Darkness Energy MEE 7', 8: 'Metal Energy MEE 8'}


def resolve(name):
    """A card key, or "Name SET" when exactly one card has it; None if unknown."""
    if name in cards:
        return name
    hits = [k for k in cards if k.rsplit(' ', 1)[0] == name]
    return hits[0] if len(hits) == 1 else None


def ported_names():
    out = subprocess.run([DIFF, '--list-ported'], capture_output=True, text=True).stdout.split('\n')
    return {l.strip() for l in out if l.strip()}


def slug(s):
    return re.sub(r'[^a-z0-9]+', '-', s.lower()).strip('-')


def is_basic_pokemon(c):
    return c['superType'] == 1 and c['stage'] == 2


def build_deck(targets, support, rng):
    deck = collections.Counter()
    by_name = collections.Counter()

    def add(n, q):
        c = cards[n]
        if c['superType'] == 3 and c.get('energyType') == 0:
            deck[n] += q
            return
        ace = 'Ace Spec' in (c.get('tags') or [])
        lim = 1 if ace else 4
        if ace and by_name['<ace>'] > 0:
            return  # one ACE SPEC per deck
        q = min(q, lim - by_name[c['name']])
        if ace and q > 0:
            by_name['<ace>'] += 1
        if q > 0:
            deck[n] += q
            by_name[c['name']] += q

    for t in targets:
        add(t, 4)
        c = cards[t]
        if c['superType'] == 1 and c['stage'] in (3, 4):
            for s in support | set(targets):
                sc = cards[s]
                if sc['superType'] == 1 and sc['name'] == c['evolvesFrom']:
                    add(s, 3)
                    if sc['stage'] == 3:
                        for s2 in support | set(targets):
                            if cards[s2]['superType'] == 1 and cards[s2]['name'] == sc['evolvesFrom']:
                                add(s2, 3)
                                break
                    break
    mons = [n for n in support if cards[n]['superType'] == 1 and is_basic_pokemon(cards[n])]
    rng.shuffle(mons)
    while sum(1 for n in deck if is_basic_pokemon(cards[n])) < 3 and mons:
        add(mons.pop(), 3)
    others = [n for n in support if cards[n]['superType'] == 2 and n not in deck]
    rng.shuffle(others)
    for n in others[:rng.randint(0, 3)]:
        add(n, rng.randint(1, 3))
    types = collections.Counter()
    for n in deck:
        c = cards[n]
        if c['superType'] == 1:
            for a in c.get('attacks') or []:
                types.update(t for t in a['cost'] if t in BASIC_ENERGY)
            types.update(t for t in c.get('cardType') or [] if t in BASIC_ENERGY)
        if c['superType'] == 3 and c.get('energyType') == 1:
            types.update(t for t in c.get('provides') or [] if t in BASIC_ENERGY)
    en = [BASIC_ENERGY[t] for t, _ in types.most_common(2)] or ['Grass Energy MEE']
    left = 60 - sum(deck.values())
    while left < 14:
        n = rng.choice([n for n in deck if n not in targets and not (cards[n]['superType'] == 3 and cards[n].get('energyType') == 0)] or list(deck))
        deck[n] -= 1
        by_name[cards[n]['name']] -= 1
        if deck[n] == 0:
            del deck[n]
        left += 1
    for i, e in enumerate(en):
        deck[e] += left // len(en) + (1 if i < left % len(en) else 0)
    return [n for n, q in sorted(deck.items()) for _ in range(q)]


def scenarios_mentioning(targets):
    """Scenario files that name a target (its key, or the card name with its set)."""
    keys = set()
    for t in targets:
        keys.add(t)
        keys.add(t.rsplit(' ', 1)[0])  # "Name SET" without the number
    out = []
    for f in sorted(glob.glob(os.path.join(ROOT, 'scenarios', '*.json'))):
        text = open(f).read()
        if any(k in text for k in keys):
            out.append(f)
    return out


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('targets', nargs='+')
    ap.add_argument('--games', type=int, default=200)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--out')
    ap.add_argument('--tag')
    ap.add_argument('--keep', action='store_true', help="keep every game's trace, not only the failing ones")
    ap.add_argument('--threads', type=int, help='worker threads (default: half the cores on a Mac, at low priority)')
    ap.add_argument('--scenario', metavar='JSON', help='run only this scenario (N games)')
    ap.add_argument('--no-scenarios', action='store_true', help="don't run the scenarios that mention the targets")
    ap.add_argument('--remote', type=int, default=0, help=argparse.SUPPRESS)  # oracle era; ignored
    args = ap.parse_args()
    if args.remote:
        print('note: --remote is ignored (the oracle is frozen; everything runs in Rust)')
    targets = [resolve(t) for t in args.targets]
    for t, k in zip(args.targets, targets):
        if k is None:
            sys.exit('unknown card: %s' % t)
    args.targets = targets
    print('targets: ' + '; '.join(args.targets))
    threads = ['--threads', str(args.threads)] if args.threads else []
    failed = False
    if args.scenario:
        rc, log = run([SCEN, args.scenario, '--games', str(args.games), '--seed', str(args.seed)] + threads)
        print(log.strip()[-6000:])
        sys.exit(rc)
    tag = args.tag or slug(args.targets[0])
    out = args.out or os.path.join(ROOT, 'corpus/cards', tag)
    os.makedirs(out, exist_ok=True)
    for f in os.listdir(out):
        fp = os.path.join(out, f)
        if os.path.isdir(fp):
            import shutil
            shutil.rmtree(fp)
        else:
            os.remove(fp)
    data = {r['key'] for r in pool if r.get('tier') == 'data'}
    support = (data | ported_names()) - set(args.targets)
    rng = random.Random(args.seed)
    decks = [{'name': '%s-%d' % (tag, k), 'cards': build_deck(args.targets, support, rng)} for k in range(6)]
    spec = {'decks': decks, 'policies': ['heur', 'random']}
    spec_path = os.path.join(out, 'spec.json.txt')
    json.dump(spec, open(spec_path, 'w'))
    rc, log = run([FUZZ, spec_path, '--games', str(args.games), '--seed', str(args.seed), '--out', out]
                  + (['--keep'] if args.keep else []) + threads)
    print(log.strip()[-4000:])
    failed |= rc != 0
    # How often the targets acted, from the same decks (scout plays its own candidate games).
    rc, log = run([SCOUT, spec_path, os.path.join(out, 'scouted.json.txt'), '--targets', '|'.join(args.targets),
                   '--candidates', str(max(args.games, 50)), '--keep', '8', '--seed', str(args.seed)])
    print(log.strip()[-2500:])
    if not args.no_scenarios:
        files = scenarios_mentioning(args.targets)
        if files:
            rc, log = run([SCEN, *files, '--games', '8', '--seed', str(args.seed), '--quiet'] + threads)
            print('scenarios mentioning the targets (%d):' % len(files))
            print(log.strip()[-4000:])
            failed |= rc != 0
        else:
            print('no scenario mentions the targets')
    sys.exit(1 if failed else 0)


if __name__ == '__main__':
    main()
