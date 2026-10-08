"""Fresh-seed self-play in the Rust engine (the oracle-free tier 4, PLAN.md
4.4; replaces tier4.py since the oracle freeze of 2026-10-08).

usage: fuzz.py [--games N] [--seed S] [--start I] [--random-decks M] [--out DIR]
               [--keep] [--threads T] [--full]

Builds the deck spec (every playable meta deck plus M random legal pool decks,
shuffled by S) and runs `engine/target/release/fuzz` on it: game i has seed
S * 100,000 + i, decks and policy (heur / random) from the seed. Engine
errors, stuck prompts, turns without legal options, invariant violations and
panics fail the run; the failing games' traces go to DIR (default
corpus/fuzz/<S>), and `--keep` keeps every trace (a corpus). The binary uses
half the cores at low priority on a Mac unless --threads / --full.
Exit status 1 on any failure.
"""
import argparse, collections, json, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIFF = os.environ.get('PTCG_DIFF') or os.path.join(ROOT, 'engine/target/release/diff')
FUZZ = os.path.join(ROOT, 'engine/target/release/fuzz')


def ported():
    out = subprocess.run([DIFF, '--list-ported'], capture_output=True, text=True).stdout.split('\n')
    return {x for x in out if x}


def random_decks(count, rng):
    """Random legal decks from every ported pool card: 3-5 Basic lines with
    their evolutions (pool or support cards), up to 14 Trainers, and Energy
    for the Pokémon's types (Special Energy included). 60 cards, max 4 copies
    of a name, at most one ACE SPEC and one Radiant."""
    cards = json.load(open(os.path.join(ROOT, 'data/cards.json')))
    pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
    support = [r['key'] for r in json.load(open(os.path.join(ROOT, 'data/support_cards.json')))]
    ok = ported()
    names = sorted({r['key'] for r in pool if r['key'] in cards and (r.get('tier') == 'data' or r['key'] in ok)})
    usable = names + [s for s in support if s in cards and s not in names]
    mons = [n for n in usable if cards[n]['superType'] == 1]
    basics = [n for n in names if n in mons and cards[n]['stage'] == 2]
    evos_of = collections.defaultdict(list)
    for n in mons:
        if cards[n].get('evolvesFrom'):
            evos_of[cards[n]['evolvesFrom']].append(n)
    trainers = [n for n in names if cards[n]['superType'] == 2]
    special = [n for n in names if cards[n]['superType'] == 3 and cards[n].get('energyType') != 0]
    # one basic Energy per type: the first printings of the Mega Evolution set (MEE 1-8)
    basic_energy = sorted(n for n, c in cards.items() if c['set'] == 'MEE' and int(c['setNumber']) <= 8 and c['superType'] == 3 and c.get('energyType') == 0)

    def is_ace(n):
        return 'Ace Spec' in (cards[n].get('tags') or [])

    def is_radiant(n):
        return 'Radiant' in (cards[n].get('tags') or []) or cards[n]['name'].startswith('Radiant ')

    out = []
    for k in range(count):
        deck = collections.Counter()
        by_name = collections.Counter()
        flags = {'ace': False, 'radiant': False}

        def add(n, q):
            c = cards[n]
            if is_ace(n):
                if flags['ace']:
                    return
                q = 1
                flags['ace'] = True
            if is_radiant(n):
                if flags['radiant']:
                    return
                q = 1
                flags['radiant'] = True
            if not (c['superType'] == 3 and c.get('energyType') == 0):
                q = min(q, 4 - by_name[c['name']])
            if q > 0:
                deck[n] += q
                by_name[c['name']] += q

        for b in rng.sample(basics, rng.randint(3, 5)):
            add(b, rng.randint(2, 4))
            s1 = evos_of.get(cards[b]['name'], [])
            if s1 and rng.random() < 0.85:
                e1 = rng.choice(s1)
                add(e1, rng.randint(2, 3))
                s2 = evos_of.get(cards[e1]['name'], [])
                if s2 and rng.random() < 0.85:
                    add(rng.choice(s2), rng.randint(1, 3))
        for t in rng.sample(trainers, rng.randint(6, 14)):
            add(t, rng.randint(1, 4))
        for s in rng.sample(special, rng.randint(0, 2)):
            add(s, rng.randint(1, 3))
        while sum(deck.values()) > 46:
            n = rng.choice(list(deck))
            deck[n] -= 1
            by_name[cards[n]['name']] -= 1
            if deck[n] == 0:
                del deck[n]
        types = collections.Counter(t for n in deck if cards[n]['superType'] == 1 for t in cards[n]['cardType'])
        en = [e for e in basic_energy if cards[e]['provides'][0] in types] or basic_energy
        en = rng.sample(en, min(len(en), 2))
        left = 60 - sum(deck.values())
        for i, e in enumerate(en):
            deck[e] += left // len(en) + (1 if i < left % len(en) else 0)
        out.append({'name': 'r%02d' % k, 'cards': [n for n, q in sorted(deck.items()) for _ in range(q)]})
    return out


def build_spec(seed, n_random):
    meta = json.load(open(os.path.join(ROOT, 'decks/meta/playable.corpus.json')))['decks']
    rng = random.Random(seed)
    decks = [dict(d, name='meta-' + d['name']) for d in meta] + random_decks(n_random, rng)
    rng.shuffle(decks)
    return {'decks': decks, 'policies': ['heur', 'random']}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--games', type=int, default=10000)
    ap.add_argument('--seed', type=int, default=1000)
    ap.add_argument('--start', type=int, default=0)
    ap.add_argument('--random-decks', type=int, default=45)
    ap.add_argument('--out')
    ap.add_argument('--keep', action='store_true')
    ap.add_argument('--threads', type=int)
    ap.add_argument('--full', action='store_true')
    a = ap.parse_args()
    out = a.out or os.path.join(ROOT, 'corpus/fuzz', str(a.seed))
    os.makedirs(out, exist_ok=True)
    spec_path = os.path.join(out, 'spec.json')
    json.dump(build_spec(a.seed, a.random_decks), open(spec_path, 'w'))
    cmd = [FUZZ, spec_path, '--games', str(a.games), '--seed', str(a.seed), '--start', str(a.start), '--out', out]
    cmd += (['--keep'] if a.keep else []) + (['--threads', str(a.threads)] if a.threads else []) + (['--full'] if a.full else [])
    sys.exit(subprocess.run(cmd).returncode)


if __name__ == '__main__':
    main()
