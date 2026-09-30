"""Coverage table (PLAN.md 1): pool cards ported / verified, meta decks playable.

usage: status.py [--md]

Ported = the card has no Twinleaf logic or has a Rust port (engine `diff
--list-ported`). Verified = listed in data/verified.json (card fullName ->
{traces, note}) after passing check_cards.py with coverage.
"""
import json, os, subprocess, sys, collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIFF = os.path.join(ROOT, 'engine/target/release/diff')


def main():
    md = '--md' in sys.argv
    pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
    ported = set(subprocess.run([DIFF, '--list-ported'], capture_output=True, text=True).stdout.split('\n'))
    vpath = os.path.join(ROOT, 'data/verified.json')
    verified = json.load(open(vpath)) if os.path.exists(vpath) else {}
    tiers = collections.Counter()
    done = collections.Counter()
    ver = collections.Counter()
    for r in pool:
        t = r.get('tier', 'absent')
        tiers[t] += 1
        if r.get('fullName') in ported:
            done[t] += 1
        if r.get('fullName') in verified:
            ver[t] += 1
    total = len(pool)
    print('## Card coverage\n' if md else 'Card coverage')
    print('| tier | pool | ported | verified |' if md else '%-8s %5s %7s %9s' % ('tier', 'pool', 'ported', 'verified'))
    if md:
        print('| --- | --- | --- | --- |')
    for t in ['data', 'short', 'custom', 'absent']:
        row = (t, tiers[t], done[t], ver[t])
        print('| %s | %d | %d | %d |' % row if md else '%-8s %5d %7d %9d' % row)
    row = ('all', total, sum(done.values()), sum(ver.values()))
    print('| **%s** | %d | %d | %d |' % row if md else '%-8s %5d %7d %9d' % row)

    arch = json.load(open(os.path.join(ROOT, 'data/meta/archetypes.json')))
    byk = {(r['set'], r['number']): r for r in pool}
    byname = collections.defaultdict(list)
    for r in pool:
        if r.get('fullName'):
            byname[r['name']].append(r)
    print('\n## Meta decks\n' if md else '\nMeta decks')
    if md:
        print('| archetype | share | cards ported | playable |')
        print('| --- | --- | --- | --- |')
    for a in arch:
        need, have, missing = 0, 0, []
        for c in a['cards']:
            r = byk.get((c['set'], str(c['number'])))
            if not r or not r.get('fullName'):
                cand = byname.get(c['name'], [])
                r = cand[0] if cand else None
            need += 1
            if r and r.get('fullName') in ported:
                have += 1
            else:
                missing.append(r['fullName'] if r else c['name'] + ' (not in Twinleaf)')
        ok = have == need
        share = a.get('share') or 0
        if md:
            print('| %s | %.1f%% | %d/%d | %s |' % (a['name'], share, have, need, 'yes' if ok else 'no'))
        else:
            print('%-32s %5.1f%% %3d/%-3d %s' % (a['name'], share, have, need, 'PLAYABLE' if ok else 'missing: ' + ', '.join(missing[:6])))


if __name__ == '__main__':
    main()
