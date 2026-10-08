"""Fetch official English card text from Limitless TCG for pool rows.

usage: fetch_official.py [--rows name-only,missing,absent|all] [--force]
Writes data/official/<SET>-<NUM>.txt (plain text of the card block) and
data/official_text.json {"SET NUM": {"name", "text", "url"}}.
"""
import html, json, os, re, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'data/official')
pool = json.load(open(os.path.join(ROOT, 'data/pool.json')))
kinds = (sys.argv[sys.argv.index('--rows') + 1] if '--rows' in sys.argv else 'name-only,missing,absent').split(',')
force = '--force' in sys.argv


def fetch(set_code, num):
    url = 'https://limitlesstcg.com/cards/%s/%s' % (set_code, num)
    req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0 (ptcg-rust card text check)'})
    with urllib.request.urlopen(req, timeout=30) as r:
        s = r.read().decode('utf-8', 'replace')
    i = s.find('class="card-text')
    if i < 0:
        return url, None
    j = s.find('Illustrated by', i)
    seg = s[i:j if j > 0 else i + 6000]
    seg = re.sub(r'<br\s*/?>|</p>|</div>', '\n', seg)
    t = html.unescape(re.sub(r'<[^>]+>', ' ', seg))
    t = '\n'.join(' '.join(l.split()) for l in t.split('\n'))
    t = re.sub(r'\n{2,}', '\n', t).strip()
    t = re.sub(r'^class="card-text[^>]*>\s*', '', t)
    return url, t


db_path = os.path.join(ROOT, 'data/official_text.json')
db = json.load(open(db_path)) if os.path.exists(db_path) else {}
for r in pool:
    match = r.get('print_match')
    if 'all' not in kinds and match not in kinds:
        continue
    key = '%s %s' % (r['set'], r['number'])
    if key in db and not force:
        continue
    try:
        url, t = fetch(r['set'], r['number'])
    except Exception as e:
        print('FAIL', key, r['name'], e)
        continue
    if not t:
        print('NOTEXT', key, r['name'], url)
        continue
    db[key] = {'name': r['name'], 'text': t, 'url': url}
    open(os.path.join(OUT, '%s-%s.txt' % (r['set'], r['number'])), 'w').write(t + '\n')
    print('ok', key, r['name'], '|', t.split('\n')[0][:70])
    time.sleep(0.5)
json.dump(db, open(db_path, 'w'), indent=1, ensure_ascii=False)
print(len(db), 'cards in', db_path)
