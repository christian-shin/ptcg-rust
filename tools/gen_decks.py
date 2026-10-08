"""Random legal decks from a tier's card pool -> decks/<prefix>-NN.txt + corpus spec.

usage: gen_decks.py <tier[,tier...]> <prefix> <count> <seed>
"""
import json, os, random, sys, collections
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
tiers = sys.argv[1].split(','); prefix = sys.argv[2]; count = int(sys.argv[3]); rng = random.Random(int(sys.argv[4]))
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
cards = json.load(open(os.path.join(ROOT, 'data/cards.json')))
rows = [r for r in pool if r.get('tier') in tiers]
names = sorted({r['key'] for r in rows})
mons = [n for n in names if cards[n]['superType'] == 1]
basics = [n for n in mons if cards[n]['stage'] == 2]
evos = [n for n in mons if cards[n]['stage'] != 2]
trainers = [n for n in names if cards[n]['superType'] == 2]
energies = [n for n in names if cards[n]['superType'] == 3 and cards[n].get('energyType') == 0]
all_basic_energy = [n for n, c in cards.items() if c['set'] == 'MEE' and int(c['setNumber']) <= 8 and c['superType'] == 3 and c.get('energyType') == 0]
decks = []
for k in range(count):
    deck = collections.Counter(); by_name = collections.Counter()
    def add(n, q):
        q = min(q, 4 - by_name[cards[n]['name']]) if cards[n]['superType'] != 3 or cards[n].get('energyType') != 0 else q
        if q > 0:
            deck[n] += q; by_name[cards[n]['name']] += q
    for n in rng.sample(basics, min(len(basics), rng.randint(3, 5))):
        add(n, rng.randint(2, 4))
        for e in evos:
            if cards[e]['evolvesFrom'] == cards[n]['name'] and rng.random() < 0.8:
                add(e, rng.randint(2, 4))
    if evos and rng.random() < 0.3:
        add(rng.choice(evos), rng.randint(1, 3))
    for n in rng.sample(trainers, min(len(trainers), rng.randint(0, 12))):
        add(n, rng.randint(1, 4))
    while sum(deck.values()) > 44:
        n = rng.choice(list(deck)); deck[n] -= 1; by_name[cards[n]['name']] -= 1
        if deck[n] == 0: del deck[n]
    types = collections.Counter(t for n in deck if cards[n]['superType'] == 1 for t in cards[n]['cardType'])
    en = [e for e in (energies or all_basic_energy) if cards[e]['provides'][0] in types] or energies or all_basic_energy
    en = rng.sample(en, min(len(en), 2))
    left = 60 - sum(deck.values())
    for i, e in enumerate(en):
        deck[e] += left // len(en) + (1 if i < left % len(en) else 0)
    assert sum(deck.values()) == 60
    path = os.path.join(ROOT, 'decks', '%s-%02d.txt' % (prefix, k))
    open(path, 'w').write(''.join('%d %s\n' % (q, n) for n, q in sorted(deck.items())))
    decks.append({'name': '%s-%02d' % (prefix, k), 'cards': [n for n, q in sorted(deck.items()) for _ in range(q)]})
spec = {'decks': decks, 'policies': ['random', 'bot', 'mix:0.2', 'mix:0.5']}
json.dump(spec, open(os.path.join(ROOT, 'decks', prefix + '.corpus.json'), 'w'))
print('wrote', len(decks), 'decks')
