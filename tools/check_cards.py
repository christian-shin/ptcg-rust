"""Verify ported cards against the oracle (PLAN.md 4.8 loop).

usage: check_cards.py "Full Name A" ["Full Name B" ...] [--games N] [--jobs J]
                      [--out DIR] [--no-gen] [--seed S] [--tag TAG]
                      [--coverage] [--min-games M] [--scout N]

Iterate on parity without --coverage (1.6x faster oracle, no 11 MB/game
coverage files); run once with --coverage at the end. --scout N plays N
candidate games in Rust and replays only the most varied target-heavy ones
in the oracle (use it to hunt branches still under --min-games).

1. Builds decks that contain every target card (4 copies, or 1 for ACE SPEC),
   filled with printed-data Pokémon and ported cards, plus basic energy that
   pays the targets' attack costs. Stage 1/2 targets need their
   pre-evolution among the targets or already ported/data cards.
2. Generates N oracle traces (bot / mixed / random policies) in parallel.
3. Replays them through the Rust `diff` tool and prints the summary.

Traces go to corpus/cards/<tag>/ (default tag: first target, slugified).
"""
import argparse, collections, json, os, random, re, subprocess, sys
import names  # tools/names.py: English keys <-> Twinleaf full names

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = os.environ.get('PTCG_ORACLE') or (os.path.join(ROOT, 'twinleaf/ptcg-server') if os.path.isdir(os.path.join(ROOT, 'twinleaf/ptcg-server/output')) else '/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server')
def _newest(*paths):
    have = [p for p in paths if os.path.exists(p)]
    return max(have, key=os.path.getmtime) if have else paths[0]


# The `iter` profile rebuilds in seconds; use whichever build is newest.
DIFF = os.environ.get('PTCG_DIFF') or _newest(os.path.join(ROOT, 'engine/target/release/diff'), os.path.join(ROOT, 'engine/target/iter/diff'))
SCOUT = _newest(os.path.join(ROOT, 'engine/target/release/scout'), os.path.join(ROOT, 'engine/target/iter/scout'))

cards = {c['fullName']: c for c in json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))}
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
BASIC_ENERGY = {1: 'Grass Energy MEE', 2: 'Fire Energy MEE', 3: 'Water Energy MEE', 4: 'Lightning Energy MEE',
                5: 'Psychic Energy MEE', 6: 'Fighting Energy MEE', 7: 'Darkness Energy MEE', 8: 'Metal Energy MEE'}


def ported_names():
    out = subprocess.run([DIFF, '--list-ported'], capture_output=True, text=True).stdout.split('\n')
    return {l.strip() for l in out if l.strip()}


def slug(s):
    return re.sub(r'[^a-z0-9]+', '-', s.lower()).strip('-')


def expand_deck(lines):
    """["4 Name", "Name", ...] -> card names (English keys accepted)."""
    out = []
    for l in lines:
        m = re.match(r'^(\d+)\s+(.+)$', l)
        n, name = (int(m.group(1)), m.group(2)) if m else (1, l)
        out += [names.twinleaf(name)] * n
    return out


def load_scenario(path):
    """Scenario JSON with every card name mapped to its Twinleaf full name."""
    sc = json.load(open(path))
    seen = []

    def one(n):
        t = names.twinleaf(n)
        seen.append(t)
        return t

    def many(v):
        out = expand_deck(v)
        seen.extend(out)
        return out

    def stack(v):
        return one(v) if isinstance(v, str) else many(v)

    for side in ('me', 'opp'):
        d = sc.get(side) or {}
        for k in ('discard', 'hand', 'deck_top', 'prizes', 'active_energy'):
            if k in d:
                d[k] = many(d[k])
        for k in ('stadium', 'active_tool'):
            if k in d:
                d[k] = one(d[k])
        if 'active' in d:
            d['active'] = stack(d['active'])
        for b in d.get('bench', []):
            b['card'] = stack(b['card'])
            if 'energy' in b:
                b['energy'] = many(b['energy'])
            if 'tool' in b:
                b['tool'] = one(b['tool'])
    for n in seen:
        if n not in cards:
            sys.exit('scenario: unknown card %s' % n)
    return sc


def scenario_decks(path):
    decks = [expand_deck(d) for d in json.load(open(path))['decks']]
    for d in decks:
        for n in d:
            if n not in cards:
                sys.exit('scenario deck: unknown card %s' % n)
    return decks


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
        ace = 'Ace Spec' in (c.get('_tags') or [])
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('targets', nargs='+')
    ap.add_argument('--games', type=int, default=16)
    ap.add_argument('--jobs', type=int, default=int(os.environ.get('PTCG_JOBS', max(1, (os.cpu_count() or 4) // 2))))
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--out')
    ap.add_argument('--tag')
    ap.add_argument('--no-gen', action='store_true')
    ap.add_argument('--coverage', action='store_true', help='record V8 block coverage per game and report per card')
    ap.add_argument('--min-games', type=int, default=3)
    ap.add_argument('--scout', type=int, default=0, help='Rust-scout N candidate games; replay the best --games of them')
    ap.add_argument('--scenario', metavar='JSON',
                    help='board edits applied at a set turn in every game (oracle scenario.ts); may also give "decks"')
    ap.add_argument('--remote', type=int, default=0, metavar='SHARDS',
                    help='play the oracle games on GitHub Actions across SHARDS runners (tools/remote_oracle.py)')
    args = ap.parse_args()
    args.targets = [names.twinleaf(t) for t in args.targets]   # English keys work too
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
        # heur: board-developing random play (bot speed without look-ahead);
        # one light bot mix keeps some realistic lines.
        spec = {'decks': decks, 'policies': ['heur', 'random', 'heur', 'mix:0.3']}
        if args.scenario:
            sc = load_scenario(args.scenario)
            if sc.pop('decks', None):
                spec['decks'] = [{'name': '%s-s%d' % (tag, k), 'cards': d} for k, d in enumerate(scenario_decks(args.scenario))]
            spec['scenario'] = sc
        spec_path = os.path.join(out, 'spec.json.txt')
        json.dump(spec, open(spec_path, 'w'))
        per = (args.games + args.jobs - 1) // args.jobs
        env = dict(os.environ)
        cov_dir = os.path.join(out, 'cov')
        if args.coverage:
            os.makedirs(cov_dir, exist_ok=True)
            env['NODE_V8_COVERAGE'] = cov_dir
        cov_files = sorted({row.get('behavior_file') or row['twinleaf_file'] for row in pool if row.get('fullName') in args.targets})
        procs = []
        if args.remote:
            sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
            import remote_oracle
            status, n, log = remote_oracle.run(spec_path, out, start=args.seed * 100000, count=args.games, shards=args.remote,
                                               cov_files=' '.join(cov_files) if args.coverage else '', tag=tag)
            bad = [l for l in log.split('\n') if 'status=error' in l or 'status=stuck' in l or 'crashed' in l]
            for l in bad[:10]:
                print('ORACLE:', l)
        elif args.scout:
            scouted = os.path.join(out, 'scouted.json.txt')
            r = subprocess.run([SCOUT, spec_path, scouted, '--targets', '|'.join(args.targets), '--candidates', str(args.scout),
                                '--keep', str(args.games), '--seed', str(args.seed)], capture_output=True, text=True)
            print(r.stderr.strip()[-1500:])
            for j in range(args.jobs):
                procs.append(subprocess.Popen(['node', 'output/oracle/cli.js', 'replay', scouted, out, str(j * per), str(per)],
                                              cwd=ORACLE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env))
        for j in range(0 if (args.scout or args.remote) else args.jobs):
            start = args.seed * 100000 + j * per
            procs.append(subprocess.Popen(['node', 'output/oracle/cli.js', 'corpus', spec_path, out, str(start), str(per)],
                                          cwd=ORACLE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env))
        logs = [p.communicate()[0] for p in procs]
        logs = logs if procs else []
        bad = [l for log in logs for l in log.split('\n') if 'status=error' in l or 'status=stuck' in l or 'crashed' in l]
        for l in bad[:10]:
            print('ORACLE:', l)
    # Twinleaf's setup rejects decks that fail DeckAnalyser.isValid (60 cards,
    # 4 copies, one ACE SPEC / Radiant, a Basic, banned pairs) by finishing
    # the game before it starts: such traces have no steps and test nothing.
    invalid = collections.Counter()
    for f in sorted(os.listdir(out)):
        if f.startswith('g') and f.endswith('.json'):
            t = json.load(open(os.path.join(out, f)))
            if not t['steps'] and t['result']['status'] == 'finished':
                invalid[' vs '.join(t['header'].get('deckNames') or ['?'])] += 1
    for k, n in invalid.items():
        print('INVALID DECK: %d game(s) %s ended before setup (Twinleaf DeckAnalyser: 60 cards, max 4 copies, '
              'one ACE SPEC, one Radiant, a Basic Pokemon)' % (n, k))
    r = subprocess.run([DIFF, out, '--quiet', '--dump', os.path.join(out, 'dump')], capture_output=True, text=True)
    print(r.stdout[-6000:])
    if args.coverage:
        files = sorted({row.get('behavior_file') or row['twinleaf_file'] for row in pool if row.get('fullName') in args.targets})
        subprocess.run([sys.executable, os.path.join(ROOT, 'tools/coverage.py'), os.path.join(out, 'cov'), *files, '--min', str(args.min_games)])
    sys.exit(r.returncode)


if __name__ == '__main__':
    main()
