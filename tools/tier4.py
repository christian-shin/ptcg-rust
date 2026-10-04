"""Tier 4 (PLAN.md 4.4, 8.2): fresh-seed self-play on both engines.

Plays oracle games with fresh seeds on a mix of meta decks and random legal
pool decks (policies random, bot, mix:0.2, mix:0.5), replays every game
through Rust with `diff` (option sets, prompts and state hash at every step,
plus the PLAN.md 4.6 invariants at every turn decision), and stops at the
first chunk with a divergence. Oracle games that end in `error` or `stuck`
also fail the run: they are card bugs that break RL rollouts.

usage: tier4.py [--games N] [--seed S] [--chunk C] [--jobs J | --remote K]
                [--random-decks M] [--out DIR] [--keep] [--no-stop]

  --seed S        1000 <= S < 42000; game seeds are S * 100,000 + i (32-bit, as
                  both engines take them), above every corpus seed (default:
                  from the clock)
  --chunk C       games per chunk (default 400); the run stops after the first
                  chunk with a failure unless --no-stop
  --remote K      play the oracle games on GitHub Actions across K runners
                  (tools/remote_oracle.py; PTCG_ORACLE_REF selects the branch)
  --out DIR       default corpus/tier4/<seed>; passing traces are deleted unless
                  --keep, failing ones stay there for `diff` and statediff

Writes <out>/summary.json and prints one line per chunk. Exit status 1 on any
divergence or oracle failure.
"""
import argparse, collections, glob, json, os, random, re, subprocess, sys, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
ORACLE = os.environ.get('PTCG_ORACLE') or os.path.join(ROOT, 'twinleaf/ptcg-server')
_bins = [os.path.join(ROOT, 'engine/target', p, 'diff') for p in ('release', 'iter')]
DIFF = os.environ.get('PTCG_DIFF') or max((b for b in _bins if os.path.exists(b)), key=os.path.getmtime)
POLICIES = ['random', 'bot', 'mix:0.2', 'mix:0.5']


def ported():
    out = subprocess.run([DIFF, '--list-ported'], capture_output=True, text=True).stdout.split('\n')
    return {x for x in out if x}


def random_decks(count, rng):
    """Random legal decks from every ported pool card: 3-5 Basic lines with
    their evolutions (pool or support cards), up to 14 Trainers, and Energy
    for the Pokémon's types (Special Energy included). 60 cards, max 4 copies
    of a name, at most one ACE SPEC and one Radiant."""
    cards = {c['fullName']: c for c in json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))}
    pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
    support = [r['fullName'] for r in json.load(open(os.path.join(ROOT, 'data/support_cards.json')))]
    ok = ported()
    names = sorted({r['fullName'] for r in pool if r.get('fullName') in cards and (r.get('tier') == 'data' or r['fullName'] in ok)})
    usable = names + [s for s in support if s in cards and s not in names]
    mons = [n for n in usable if cards[n]['superType'] == 1]
    basics = [n for n in names if n in mons and cards[n]['stage'] == 2]
    evos_of = collections.defaultdict(list)
    for n in mons:
        if cards[n].get('evolvesFrom'):
            evos_of[cards[n]['evolvesFrom']].append(n)
    trainers = [n for n in names if cards[n]['superType'] == 2]
    special = [n for n in names if cards[n]['superType'] == 3 and cards[n].get('energyType') != 0]
    basic_energy = sorted(n for n, c in cards.items() if n.endswith(' MEE') and c['superType'] == 3 and c.get('energyType') == 0)

    def is_ace(n):
        return 'Ace Spec' in (cards[n].get('_tags') or [])

    def is_radiant(n):
        return 'Radiant' in (cards[n].get('_tags') or []) or cards[n]['name'].startswith('Radiant ')

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
    meta = json.load(open(os.path.join(ROOT, 'decks/meta-tl/playable.corpus.json')))['decks']
    rng = random.Random(seed)
    decks = [dict(d, name='meta-' + d['name']) for d in meta] + random_decks(n_random, rng)
    rng.shuffle(decks)
    return {'decks': decks, 'policies': POLICIES}


def play_local(spec_path, out, start, count, jobs):
    per = (count + jobs - 1) // jobs
    procs = []
    for j in range(jobs):
        s, c = start + j * per, min(per, start + count - (start + j * per))
        if c <= 0:
            break
        procs.append(subprocess.Popen(['node', 'output/oracle/cli.js', 'corpus', spec_path, out, str(s), str(c)],
                                      cwd=ORACLE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True))
    log = ''.join(p.communicate()[0] for p in procs)
    return log


def oracle_failures(out, log):
    bad = []
    for l in log.split('\n'):
        if 'crashed' in l:
            bad.append(('crashed', l.strip()))
    for f in sorted(glob.glob(os.path.join(out, 'g*.json'))):
        with open(f) as fh:
            s = fh.read()
        i = s.rfind('"result"')
        r = json.loads('{' + s[i:])['result'] if i >= 0 else {}
        if r.get('status') in ('error', 'stuck'):
            bad.append((r['status'], '%s: %s' % (os.path.basename(f), r.get('message', ''))))
    return bad


def replay(out):
    p = subprocess.run([DIFF, out, '--quiet'], capture_output=True, text=True)
    div = re.findall(r'DIVERGED (\S+) at step (\d+): (\S+)\n  ([^\n]*)', p.stdout)
    m = re.search(r'(\d+) traces: (\d+) pass \((\d+) steps\), (\d+) diverged, (\d+) unsupported, (\d+) approved', p.stdout)
    stats = dict(zip(['traces', 'pass', 'steps', 'diverged', 'unsupported', 'approved'], map(int, m.groups()))) if m else {}
    return stats, div


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--games', type=int, default=1000)
    ap.add_argument('--seed', type=int, default=1000 + int(time.time()) % 41000)
    ap.add_argument('--chunk', type=int, default=400)
    ap.add_argument('--jobs', type=int, default=max(1, (os.cpu_count() or 4) - 1))
    ap.add_argument('--remote', type=int, default=0)
    ap.add_argument('--random-decks', type=int, default=45)
    ap.add_argument('--out')
    ap.add_argument('--keep', action='store_true')
    ap.add_argument('--no-stop', action='store_true')
    a = ap.parse_args()
    out = a.out or os.path.join(ROOT, 'corpus/tier4', str(a.seed))
    os.makedirs(out, exist_ok=True)
    spec = build_spec(a.seed, a.random_decks)
    spec_path = os.path.join(out, 'spec.json.txt')
    json.dump(spec, open(spec_path, 'w'))
    if not 1000 <= a.seed < 42000 or a.games > 100000:
        sys.exit('--seed must be in [1000, 42000) and --games <= 100000 (32-bit seeds)')
    base = a.seed * 100000
    tot = collections.Counter()
    failures = []
    t0 = time.time()
    done = 0
    while done < a.games:
        n = min(a.chunk, a.games - done)
        chunk = os.path.join(out, 'c%06d' % (done // a.chunk))
        os.makedirs(chunk, exist_ok=True)
        if a.remote:
            import remote_oracle
            _, _, log = remote_oracle.run(spec_path, chunk, start=base + done, count=n, shards=a.remote,
                                          tag='tier4-%d' % a.seed, ref=os.environ.get('PTCG_ORACLE_REF', 'oracle'))
        else:
            log = play_local(spec_path, chunk, base + done, n, a.jobs)
        bad = oracle_failures(chunk, log)
        stats, div = replay(chunk)
        for k, v in stats.items():
            tot[k] += v
        tot['oracle_failures'] += len(bad)
        failures += [{'kind': 'oracle-' + k, 'detail': d, 'chunk': chunk} for k, d in bad]
        failures += [{'kind': 'diverged-' + w, 'trace': t, 'step': int(s), 'detail': d} for t, s, w, d in div]
        done += n
        print('[tier4 seed %d] %d/%d games, %.0fs: %s; oracle failures %d; diverged %d' % (
            a.seed, done, a.games, time.time() - t0, stats, len(bad), len(div)), flush=True)
        for k, d in bad[:5]:
            print('  ORACLE %s %s' % (k, d))
        for t, s, w, d in div[:5]:
            print('  DIVERGED %s at step %s: %s %s' % (t, s, w, d))
        if not a.keep:
            failing = {os.path.basename(t) for t, *_ in div} | {d.split(':')[0] for _, d in bad}
            for f in glob.glob(os.path.join(chunk, 'g*.json')):
                if os.path.basename(f) not in failing:
                    os.remove(f)
        if (bad or div) and not a.no_stop:
            break
    summary = {'seed': a.seed, 'games': done, 'stats': dict(tot), 'failures': failures, 'seconds': round(time.time() - t0)}
    json.dump(summary, open(os.path.join(out, 'summary.json'), 'w'), indent=1)
    ok = not failures and tot['traces'] == done and tot['unsupported'] == 0
    print('[tier4 seed %d] %s: %d games, %d steps, %d failures' % (a.seed, 'GREEN' if ok else 'RED', done, tot['steps'], len(failures)))
    sys.exit(0 if ok else 1)


if __name__ == '__main__':
    main()
