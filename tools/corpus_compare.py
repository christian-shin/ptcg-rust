"""Compare two Rust-recorded corpora of the same seeds, game by game, ignoring
names and hashes: did each game make the same decisions and get the same
chance outcomes and result?

usage: corpus_compare.py OLD_DIR NEW_DIR [--show N]

Per game (matched by file name) it compares the result (status, winner,
turn), the number of steps, and at every step the deciding player, the
decision kind, the prompt class and raw prompt answer, the number of turn
options and the chosen action's kind, card instance and target, and the
chance outcomes drawn. Card ids are compared by instance (`#n`), so a change
of set code or card name doesn't count; attack and Ability names are not
compared. Use it after a change that should only rename things (every game
the same), or after a rules change to list the games it changed. Prints the
first difference of the first N changed games and the cards their decks share
most; exit status 1 when any game differs.
"""
import collections, glob, json, os, sys


def inst(card):
    return str(card).rsplit('#', 1)[-1] if card is not None else None


def turn_key(a):
    return (a.get('a'), inst(a.get('card')), json.dumps(a.get('target'), sort_keys=True), json.dumps(a.get('source'), sort_keys=True),
            a.get('bench'), a.get('index'))


def first_diff(old, new):
    ro, rn = old['result'], new['result']
    for k in ('status', 'winner', 'turn'):
        if ro.get(k) != rn.get(k):
            res = 'result %s %r -> %r' % (k, ro.get(k), rn.get(k))
            break
    else:
        res = None
    so, sn = old['steps'], new['steps']
    if json.dumps(old['start'].get('c')) != json.dumps(new['start'].get('c')):
        return 'start chance'
    for i, (a, b) in enumerate(zip(so, sn)):
        if a['p'] != b['p'] or a['d'].get('kind') != b['d'].get('kind'):
            return 'step %d: decision %s/%s -> %s/%s' % (i, a['p'], a['d'].get('kind'), b['p'], b['d'].get('kind'))
        if a['d'].get('kind') == 'prompt':
            if a['d'].get('cls') != b['d'].get('cls'):
                return 'step %d: prompt %s -> %s' % (i, a['d'].get('cls'), b['d'].get('cls'))
            if json.dumps(a['a'], sort_keys=True) != json.dumps(b['a'], sort_keys=True):
                return 'step %d: %s answer %s -> %s' % (i, a['d'].get('cls'), json.dumps(a['a'])[:80], json.dumps(b['a'])[:80])
        else:
            if len(a['d'].get('options', [])) != len(b['d'].get('options', [])):
                return 'step %d: %d turn options -> %d' % (i, len(a['d']['options']), len(b['d']['options']))
            if turn_key(a['a']) != turn_key(b['a']):
                return 'step %d: turn %s -> %s' % (i, json.dumps(a['a'])[:90], json.dumps(b['a'])[:90])
        if json.dumps(a['c']) != json.dumps(b['c']):
            return 'step %d: chance differs' % i
    if len(so) != len(sn):
        return 'steps %d -> %d' % (len(so), len(sn))
    return res


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    show = int(sys.argv[sys.argv.index('--show') + 1]) if '--show' in sys.argv else 10
    if len(args) < 2:
        sys.exit(__doc__)
    old_dir, new_dir = args[0], args[1]
    names = sorted(os.path.basename(f) for f in glob.glob(os.path.join(old_dir, 'g*.json')))
    changed, missing = [], 0
    for n in names:
        nf = os.path.join(new_dir, n)
        if not os.path.exists(nf):
            missing += 1
            continue
        old, new = json.load(open(os.path.join(old_dir, n))), json.load(open(nf))
        d = first_diff(old, new)
        if d:
            changed.append((n, d, new['header']['decks']))
    print('%d games compared, %d changed, %d missing in %s' % (len(names) - missing, len(changed), missing, new_dir))
    for n, d, _ in changed[:show]:
        print('  %s: %s' % (n, d))
    if changed:
        common = collections.Counter()
        for _, _, decks in changed:
            common.update(set(decks[0]) | set(decks[1]))
        print('cards most common in the changed games:')
        for c, k in common.most_common(12):
            print('  %4d  %s' % (k, c))
    sys.exit(1 if changed or missing else 0)


if __name__ == '__main__':
    main()
