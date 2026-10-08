"""Rebuild data/meta/* from the current INTERNATIONAL Standard metagame on Limitless.

usage: fetch_meta.py [--refresh] [--top N] [--sleep SECONDS]

Outputs (all under data/meta/):
  archetypes.json   top N (default 20) archetypes: name, share, source_url, archetype_url,
                    tournament, country, placement, cards[{count,name,set,number}]
  card_usage.json   "SET NUMBER" -> {name, decks, weighted}, sorted by weighted desc
  port_order.json   cards to port, most valuable first (see below)
  unmatched.txt     decklist cards whose (set, number) is not in data/pool.json

Sources (the same filter the old hand-built file used; it reproduces the old shares):
  https://limitlesstcg.com/decks?format=standard&time=3months
      international Standard ("Standard (JP)" is a separate format and is never fetched)
  https://limitlesstcg.com/tournaments?format=standard&time=3months
      the international tournaments in that window and their country flag
  https://limitlesstcg.com/decks/<id>/results?format=standard&time=3months
      every placement of an archetype in those tournaments (one request per archetype)
  https://limitlesstcg.com/decks/list/<id>        the decklist itself

International only: only tournaments whose country is not JP and whose name is not a
Japanese event (Champions League, City League, ...) are considered, and only the Masters
division (the "(SR)"/"(JR)" sections are skipped). Indonesia Premier Ball League and other
non-Japanese events count. All card codes are Limitless' international set codes.

List choice: candidates are sorted by placement (1st first), ties broken by the most recent
tournament. The first list whose every card is in data/pool.json is used; "in the pool"
means (set, number) is a pool row, or the card name is in the pool under another printing
(the same fallback status.py and meta_decks.py use). Lists with a card outside the pool
are skipped and the next one is tried.

Politeness: one request every --sleep seconds (default 1.5); raw HTML is cached in
$FETCH_META_CACHE or <tmp>/pkmntcg-limitless-cache (--refresh ignores the cache).

card_usage.json
  decks    = number of the archetypes' chosen lists that contain the card (any copy count)
  weighted = sum of the shares (%) of those archetypes (copies do not matter), rounded to
             2 decimals. Checked against the previous hand-built file: Lillie's
             Determination in 17 lists -> 87.59 = sum of those 17 shares.
  Keys are the decklist's own "SET NUMBER"; the file is sorted by weighted desc.

port_order.json
  Each decklist card is resolved to a pool row (exact set+number, else the first pool row
  with the same name). Rows are merged per resolved card:
  weighted = sum of the shares of the archetypes whose list contains it (an archetype is
  counted once even if two printings appear), decks = how many archetypes. Sorted by
  weighted desc (ties: first seen in share order). Entry:
    {key (pool.json `key`, international name + set + number), weighted, decks,
     tier (pool.json tier)}
  Port the top of the list first: the cards with the highest share-weighted coverage.
"""
import argparse, collections, datetime, hashlib, html, json, os, re, sys, tempfile, time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = 'https://limitlesstcg.com'
FILTER = 'format=standard&time=3months'
UA = 'Mozilla/5.0 (pkmntcg meta-deck research; low volume)'
JAPAN_NAME = re.compile(r'champions league|city league|japan|yokohama|osaka|kyoto|nagoya|sapporo|fukuoka|kobe|sendai|'
                        r'hiroshima|chiba|saitama|tokyo|\bJP\b', re.I)
JAPAN_SETS = {'M1L', 'M1S', 'M2', 'M2a', 'M3', 'M4', 'M5', 'M6', 'SV11B', 'SV11W'}
CACHE = os.environ.get('FETCH_META_CACHE') or os.path.join(tempfile.gettempdir(), 'pkmntcg-limitless-cache')
_last = [0.0]


def get(path, refresh=False, sleep=1.5):
    os.makedirs(CACHE, exist_ok=True)
    f = os.path.join(CACHE, hashlib.md5(path.encode()).hexdigest() + '.html')
    if os.path.exists(f) and not refresh:
        return open(f, encoding='utf-8').read()
    wait = _last[0] + sleep - time.time()
    if wait > 0:
        time.sleep(wait)
    req = urllib.request.Request(BASE + path, headers={'User-Agent': UA})
    for attempt in range(3):
        try:
            txt = urllib.request.urlopen(req, timeout=60).read().decode('utf-8')
            break
        except Exception as e:
            if attempt == 2:
                raise
            print('retry', path, e, file=sys.stderr)
            time.sleep(5 * (attempt + 1))
    _last[0] = time.time()
    open(f, 'w', encoding='utf-8').write(txt)
    return txt


def text(s):
    return html.unescape(re.sub(r'\s+', ' ', re.sub(r'<[^>]*>', '', s))).strip()


def parse_archetypes(page, top):
    out = []
    for m in re.finditer(r'<a href="/decks/(\d+)">(.*?)</a></td>\s*<td>[\d.]+</td>\s*<td>([\d.]+)%</td>', page, re.S):
        out.append({'id': m.group(1), 'name': text(m.group(2)), 'share': float(m.group(3))})
    return out[:top]


def parse_tournaments(page):
    """id -> {country, name} for tournaments listed under the international filter."""
    out = {}
    for m in re.finditer(r'data-country="(\w*)"\s*data-name="([^"]*)".*?href="/tournaments/(\d+)"', page, re.S):
        out[m.group(3)] = {'country': m.group(1), 'name': html.unescape(m.group(2))}
    return out


def tournament_country(tid, known, refresh, sleep):
    if tid in known:
        return known[tid]['country']
    pg = get('/tournaments/%s' % tid, refresh, sleep)
    m = re.search(r'class="infobox-heading".*?alt="(\w\w)"', pg, re.S)
    known[tid] = {'country': m.group(1) if m else '', 'name': ''}
    return known[tid]['country']


def parse_results(page):
    """-> [(tournament_id, tournament title, placement, list_id)] for Masters, standard rows."""
    rows = []
    for sec in re.split(r'<th class="sub-heading"', page)[1:]:
        h = re.search(r'href="/tournaments/(\d+)(/\w+)?">([^<]*)<', sec)
        if not h or h.group(2):          # /SR or /JR section
            continue
        title = html.unescape(h.group(3)).strip()
        if re.search(r'\((SR|JR)\)\s*$', title):
            continue
        for r in re.split(r'<tr>', sec)[1:]:
            pl = re.search(r'<td>(\d+)(?:st|nd|rd|th)</td>', r)
            li = re.search(r'/decks/list/(\d+)', r)
            fmt = re.search(r'class="format"[^>]*alt="([^"]*)"', r)
            if pl and li and (not fmt or fmt.group(1) == 'standard'):
                rows.append((h.group(1), title, int(pl.group(1)), li.group(1)))
    return rows


def tdate(title):
    m = re.match(r'(\d+)\w* (\w+) (\d{4})', title)
    try:
        return datetime.datetime.strptime('%s %s %s' % m.groups(), '%d %B %Y').date().isoformat()
    except Exception:
        return '0000-00-00'


def parse_list(page):
    cards = []
    for m in re.finditer(r'<div class="decklist-card" data-set="(\w+)" data-number="(\w+)".*?'
                         r'<span class="card-count">(\d+)</span>\s*<span class="card-name">([^<]*)</span>', page, re.S):
        cards.append({'count': int(m.group(3)), 'name': html.unescape(m.group(4)).strip(),
                      'set': m.group(1), 'number': m.group(2)})
    return cards


def load_pool():
    pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
    keys = {(r['set'], r['number']) for r in pool}
    names = collections.defaultdict(list)
    for r in pool:
        names[r['name']].append((r['set'], r['number']))
    byk = {(r['set'], r['number']): r for r in pool}
    byname = collections.defaultdict(list)
    for r in pool:
        byname[r['name']].append(r)
    return keys, names, byk, byname


def in_pool(c, keys, names):
    return (c['set'], c['number']) in keys or c['name'] in names


def resolve(c, byk, byname):
    """Pool row for a decklist card, as status.py / meta_decks.py do."""
    r = byk.get((c['set'], c['number']))
    if not r:
        cand = byname.get(c['name'], [])
        r = cand[0] if cand else r
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--refresh', action='store_true')
    ap.add_argument('--top', type=int, default=20)
    ap.add_argument('--sleep', type=float, default=1.5)
    a = ap.parse_args()
    R = lambda p: get(p, a.refresh, a.sleep)
    keys, names, byk, byname = load_pool()
    archs = parse_archetypes(R('/decks?' + FILTER), a.top)
    tours = parse_tournaments(R('/tournaments?' + FILTER))
    print('archetypes:', [(x['name'], x['share']) for x in archs])
    print('international tournaments:', {k: (v['name'], v['country']) for k, v in tours.items()})
    result, missing = [], []
    for x in archs:
        rows = parse_results(R('/decks/%s/results?%s' % (x['id'], FILTER)))
        cands = []
        for tid, title, place, lid in rows:
            country = tournament_country(tid, tours, a.refresh, a.sleep)
            if country == 'JP' or JAPAN_NAME.search(title):
                continue
            cands.append((place, tdate(title), tid, title, country, lid))
        cands.sort(key=lambda c: c[1], reverse=True)   # newest first
        cands.sort(key=lambda c: c[0])                 # stable: placement, then newest
        chosen = None
        for place, _, tid, title, country, lid in cands:
            cards = parse_list(R('/decks/list/' + lid))
            total = sum(c['count'] for c in cards)
            bad = [c for c in cards if not in_pool(c, keys, names) or c['set'] in JAPAN_SETS]
            if total != 60 or bad:
                print('  skip %s list %s (%s #%d): %s' % (x['name'], lid, title, place,
                      'size %d' % total if total != 60 else ', '.join('%s %s %s' % (c['name'], c['set'], c['number']) for c in bad)))
                continue
            chosen = {'name': x['name'], 'share': x['share'], 'source_url': '%s/decks/list/%s' % (BASE, lid),
                      'archetype_url': '%s/decks/%s' % (BASE, x['id']), 'tournament': title,
                      'country': country, 'placement': place, 'cards': cards}
            break
        if chosen:
            result.append(chosen)
            print('%-32s %5.2f%%  %s #%d (%s)' % (x['name'], x['share'], chosen['tournament'], chosen['placement'], chosen['country']))
        else:
            missing.append(x['name'])
            print('NO INTERNATIONAL LIST IN POOL for', x['name'], file=sys.stderr)

    meta = os.path.join(ROOT, 'data/meta')
    json.dump(result, open(os.path.join(meta, 'archetypes.json'), 'w', encoding='utf-8'), indent=1, ensure_ascii=False)

    # card_usage.json
    usage = collections.OrderedDict()
    for ar in result:
        for c in ar['cards']:
            k = '%s %s' % (c['set'], c['number'])
            u = usage.setdefault(k, {'name': c['name'], 'decks': 0, 'weighted': 0.0})
            u['decks'] += 1
            u['weighted'] += ar['share']
    for u in usage.values():
        u['weighted'] = round(u['weighted'], 2)
    usage = collections.OrderedDict(sorted(usage.items(), key=lambda kv: -kv[1]['weighted']))
    json.dump(usage, open(os.path.join(meta, 'card_usage.json'), 'w', encoding='utf-8'), indent=1, ensure_ascii=False)

    # port_order.json: merge per resolved pool card
    merged = collections.OrderedDict()
    for ar in result:
        seen = set()
        for c in ar['cards']:
            r = resolve(c, byk, byname)
            mk = r['key'] if r else '%s %s %s' % (c['name'], c['set'], c['number'])
            if mk in seen:
                continue
            seen.add(mk)
            e = merged.setdefault(mk, {'key': mk, 'weighted': 0.0,
                                       'decks': 0, 'tier': r.get('tier') if r else None})
            e['weighted'] += ar['share']
            e['decks'] += 1
    order = sorted(merged.values(), key=lambda e: -round(e['weighted'], 2))
    for e in order:
        e['weighted'] = round(e['weighted'], 2)
    json.dump(order, open(os.path.join(meta, 'port_order.json'), 'w', encoding='utf-8'), indent=1, ensure_ascii=False)

    # unmatched.txt
    lines, nname, nnone = [], 0, 0
    for k, u in usage.items():
        s, n = k.split(' ', 1)
        if (s, n) in keys:
            continue
        alt = names.get(u['name'])
        if alt:
            nname += 1
            lines.append('%s\t%s\tNAME-MATCH other set -> %s %s' % (k, u['name'], alt[0][0], alt[0][1]))
        else:
            nnone += 1
            lines.append('%s\t%s\tNOT IN POOL (decks=%d, weighted=%s)' % (k, u['name'], u['decks'], u['weighted']))
    with open(os.path.join(meta, 'unmatched.txt'), 'w', encoding='utf-8') as f:
        f.write('# Decklist cards not found in data/pool.json by (set, number). %d entries: %d matched by name, %d not in pool.\n'
                % (len(lines), nname, nnone))
        f.write('# columns: SET NUMBER\tname\tresolution\n')
        f.write(''.join(l + '\n' for l in lines))
    print('wrote %d archetypes, %d usage keys, %d port entries; no list for: %s' % (len(result), len(usage), len(order), missing or 'none'))


if __name__ == '__main__':
    main()
