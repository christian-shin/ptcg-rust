"""Fetch the Pokémon TCG Rulings Compendium (compendium.pokegym.net) and map
rulings to pool cards.

The Compendium is the judges' database of official rulings (TPCi Rules Team).
Every ruling is tagged with topics such as "Trainers » Ultra Ball",
"Abilities » Wild Growth" or "Trainers » *Trainers in General".

usage: fetch_rulings.py [--refetch]
writes data/rulings/all.json       every ruling: {id, topics, q, a, source}
       data/rulings/cards.json     pool key -> ruling ids whose topic names the
                                   card (Trainer/Energy name) or one of its
                                   attacks/Abilities (official names; an attack
                                   name shared by several cards matches all of them)
       data/rulings/general.json   ruling ids of Meta-Rulings, Gameplay and the
                                   "*... in General" topics
Pages are cached in data/rulings/cache/ (local, git-ignored).
"""
import html, json, os, re, sys, time, unicodedata, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'data/rulings')
CACHE = os.path.join(OUT, 'cache')
BASE = 'https://compendium.pokegym.net/category/%s/'
CATEGORIES = ['1-errata', '2-meta-rulings', '3-attacks', '4-abilities', '5-trainers', '6-energy', '7-gameplay']
ART = re.compile(r'<article id="post-(\d+)".*?</article>', re.S)


def fetch(url, path):
    if os.path.exists(path) and '--refetch' not in sys.argv:
        return open(path, encoding='utf-8').read()
    req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0 (ptcg-rust rules engine; rulings lookup)'})
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            s = r.read().decode('utf-8', 'replace')
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return None
        raise
    # Keep only the articles: the pages carry a ~1.4 MB category sidebar.
    s = '\n'.join(m.group(0) for m in ART.finditer(s))
    open(path, 'w', encoding='utf-8').write(s)
    time.sleep(0.5)
    return s


def text(h):
    return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', ' ', h))).strip()


def parse(s):
    out = []
    for m in ART.finditer(s):
        a = m.group(0)
        cats = re.search(r'<div class="ruling-categories">(.*?)</div>', a, re.S)
        topics = [text(t) for t in (cats.group(1).split('|') if cats else [])]
        qa = re.findall(r'<dt>(.*?)</dt>\s*<dd>(.*?)</dd>', a, re.S)
        src = re.search(r'<div id="source">(.*?)</div>', a, re.S)
        for q, ans in qa or [('', '')]:
            out.append({'id': int(m.group(1)), 'topics': topics, 'q': text(q), 'a': text(ans),
                        'source': text(src.group(1)).replace('Source: ', '') if src else ''})
    return out


def norm(s):
    s = unicodedata.normalize('NFKD', s or '').encode('ascii', 'ignore').decode().lower()
    return re.sub(r'[^a-z0-9]+', ' ', s).strip()


def main():
    os.makedirs(CACHE, exist_ok=True)
    rulings = {}
    for c in CATEGORIES:
        page = 1
        while True:
            url = BASE % c + ('page/%d/' % page if page > 1 else '')
            s = fetch(url, os.path.join(CACHE, '%s-%03d.html' % (c, page)))
            if not s:
                break
            got = parse(s)
            if not got:
                break
            for r in got:
                rulings.setdefault('%d:%s' % (r['id'], r['q'][:40]), r)
            print('%s page %d: %d rulings (total %d)' % (c, page, len(got), len(rulings)), flush=True)
            page += 1
    allr = sorted(rulings.values(), key=lambda r: r['id'])
    for i, r in enumerate(allr):
        r['n'] = i
    json.dump(allr, open(os.path.join(OUT, 'all.json'), 'w'), indent=1, ensure_ascii=False)

    # Topic name -> rulings.
    by_topic = {}
    general = set()
    for r in allr:
        for t in r['topics']:
            parts = [p.strip() for p in t.split('»')]
            if len(parts) == 2:
                by_topic.setdefault(norm(parts[1]), set()).add(r['n'])
                if parts[1].startswith('*'):
                    general.add(r['n'])
            if parts and parts[0] in ('Meta-Rulings', 'Gameplay', 'Errata'):
                general.add(r['n'])
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from text_audit import parse as parse_official   # attacks and Abilities from the official text
    official = json.load(open(os.path.join(ROOT, 'data/official_text.json')))
    pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
    cards = {}
    for row in pool:
        key = '%s %s' % (row['set'], row['number'])
        if key not in official:
            continue
        o = parse_official(official[key]['text'])
        names = [official[key]['name']] + [a['name'] for a in o['attacks']] + o['abilities']
        ids = set()
        for n in names:
            ids |= by_topic.get(norm(n), set())
        if ids:
            cards[row['key']] = sorted(ids)
    json.dump(cards, open(os.path.join(OUT, 'cards.json'), 'w'), indent=1, ensure_ascii=False)
    json.dump(sorted(general), open(os.path.join(OUT, 'general.json'), 'w'))
    print('%d rulings; %d pool cards have card rulings; %d general rulings' % (len(allr), len(cards), len(general)))


if __name__ == '__main__':
    main()
