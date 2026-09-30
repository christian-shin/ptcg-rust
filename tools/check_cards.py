"""Verify ported cards against the oracle (PLAN.md 4.8 loop).

usage: check_cards.py "Full Name A" ["Full Name B" ...] [--games N] [--jobs J]
                      [--out DIR] [--no-gen] [--seed S] [--tag TAG]

1. Builds decks that contain every target card (4 copies, or 1 for ACE SPEC),
   filled with printed-data Pokémon and ported cards, plus basic energy that
   pays the targets' attack costs. Stage 1/2 targets need their
   pre-evolution among the targets or already ported/data cards.
2. Generates N oracle traces (bot / mixed / random policies) in parallel.
3. Replays them through the Rust `diff` tool and prints the summary.

Traces go to corpus/cards/<tag>/ (default tag: first target, slugified).
"""
import argparse, collections, json, os, random, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = os.environ.get('PTCG_ORACLE') or (os.path.join(ROOT, 'twinleaf/ptcg-server') if os.path.isdir(os.path.join(ROOT, 'twinleaf/ptcg-server/output')) else '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server')
DIFF = os.path.join(ROOT, 'engine/target/release/diff')

cards = {c['fullName']: c for c in json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))}
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
BASIC_ENERGY = {1: 'Grass Energy MEE', 2: 'Fire Energy MEE', 3: 'Water Energy MEE', 4: 'Lightning Energy MEE',
                5: 'Psychic Energy MEE', 6: 'Fighting Energy MEE', 7: 'Darkness Energy MEE', 8: 'Metal Energy MEE'}


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
        lim = 1 if 'Ace Spec' in (c.get('_tags') or []) else 4
        q = min(q, lim - by_name[c['name']])
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('targets', nargs='+')
    ap.add_argument('--games', type=int, default=24)
    ap.add_argument('--jobs', type=int, default=4)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--out')
    ap.add_argument('--tag')
    ap.add_argument('--no-gen', action='store_true')
    ap.add_argument('--coverage', action='store_true', help='record V8 block coverage per game and report per card')
    ap.add_argument('--min-games', type=int, default=10)
    args = ap.parse_args()
    for t in args.targets:
        if t not in cards:
            sys.exit('unknown card: %s' % t)
    tag = args.tag or slug(args.targets[0])
    out = args.out or os.path.join(ROOT, 'corpus/cards', tag)
    if not args.no_gen:
        os.makedirs(out, exist_ok=True)
        for f in os.listdir(out):
            fp = os.path.join(out, f)
            if os.path.isdir(fp):
                import shutil
                shutil.rmtree(fp)
            else:
                os.remove(fp)
        data = {r['fullName'] for r in pool if r.get('tier') == 'data'}
        support = (data | ported_names()) - set(args.targets)
        rng = random.Random(args.seed)
        decks = []
        for k in range(6):
            decks.append({'name': '%s-%d' % (tag, k), 'cards': build_deck(args.targets, support, rng)})
        # Opponents: half target decks, half plain support decks.
        spec = {'decks': decks, 'policies': ['bot', 'mix:0.3', 'mix:0.6', 'random']}
        spec_path = os.path.join(out, 'spec.json.txt')
        json.dump(spec, open(spec_path, 'w'))
        per = (args.games + args.jobs - 1) // args.jobs
        env = dict(os.environ)
        cov_dir = os.path.join(out, 'cov')
        if args.coverage:
            os.makedirs(cov_dir, exist_ok=True)
            env['NODE_V8_COVERAGE'] = cov_dir
        procs = []
        for j in range(args.jobs):
            start = args.seed * 100000 + j * per
            procs.append(subprocess.Popen(['node', 'output/oracle/cli.js', 'corpus', spec_path, out, str(start), str(per)],
                                          cwd=ORACLE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env))
        logs = [p.communicate()[0] for p in procs]
        bad = [l for log in logs for l in log.split('\n') if 'status=error' in l or 'status=stuck' in l or 'crashed' in l]
        for l in bad[:10]:
            print('ORACLE:', l)
    r = subprocess.run([DIFF, out, '--quiet', '--dump', os.path.join(out, 'dump')], capture_output=True, text=True)
    print(r.stdout[-6000:])
    if args.coverage:
        files = sorted({row['twinleaf_file'] for row in pool if row.get('fullName') in args.targets})
        subprocess.run([sys.executable, os.path.join(ROOT, 'tools/coverage.py'), os.path.join(out, 'cov'), *files, '--min', str(args.min_games)])
    sys.exit(r.returncode)


if __name__ == '__main__':
    main()
