#!/usr/bin/env python3
"""Collect the official Japanese FAQ (pokemon-card.com) Q&As for every card in the pool.

  tools/jp_faq.py [--dry NAME ...] [--refresh] [--limit N]
                  [--csv-dir DIR] [--cache DIR] [--out-dir DIR]

English pool names -> Japanese names through the two Kaggle playground card-list CSVs
(same Card ID = same printing). Each distinct Japanese name is searched (full name, then the
name without ex/V/VSTAR/VMAX if the full name gave nothing); at most 1 request/second.
Writes docs/rulings/jp-faq.json and jp-faq.md. Python 3 stdlib only.
"""
import argparse, csv, datetime, html, json, os, re, sys, time, unicodedata
import urllib.parse, urllib.request, urllib.error

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEF_CSV = os.path.expanduser(
    "~/Downloads/the-pokemon-company-ptcg-ai-battle-challenge-playground")
DEF_CACHE = ("/private/tmp/claude-501/-Users-christianshin-Documents-pkmntcg/"
             "9deb2769-00b0-461c-86c1-8bcbc72e4a7c/scratchpad/jp_faq_html")
URL = ("https://www.pokemon-card.com/rules/faq/search.php?freeword={w}"
       "&regulation_faq_main_item1=all&regulation=all&page={p}")
UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 "
      "(KHTML, like Gecko) Version/17.0 Safari/605.1.15")

# cabt's names for type Special Energy / basic Energy -> our pool names
HAND = {
    "Grow Grass Energy": "Growing Grass Energy",
    "Telepath Psychic Energy": "Telepathic Psychic Energy",
    "Rock Fighting Energy": "Rocky Fighting Energy",
    "Bubbly {W} Energy": "Bubbly Water Energy",
    "Magnetic {M} Energy": "Magnetic Metal Energy",
    "Voltaic {L} Energy": "Voltaic Lightning Energy",
    "Shadowy {D} Energy": "Shadowy Darkness Energy",
    "Nitro {R} Energy": "Nitro Fire Energy",
}
BASIC = {"G": "Grass", "R": "Fire", "W": "Water", "L": "Lightning", "P": "Psychic",
         "F": "Fighting", "D": "Darkness", "M": "Metal"}


def norm(s):
    return unicodedata.normalize("NFKC", s).replace("’", "'").strip().lower()


def en_alias(name):
    if name in HAND:
        return HAND[name]
    m = re.fullmatch(r"Basic \{(\w)\} Energy", name)
    if m:
        return BASIC[m.group(1)] + " Energy"
    return name


def read_csv(path):
    with open(path, encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def jp_names(csv_dir):
    en = read_csv(os.path.join(csv_dir, "EN_Card_Data_R2_full.csv"))
    jp = read_csv(os.path.join(csv_dir, "JP_Card_Data_R2_full.csv"))
    jp_by_id = {r["カード ID"]: r["カード名"] for r in jp}
    out = {}  # normalized EN name -> ordered set of JP names
    for r in en:
        j = jp_by_id.get(r["Card ID"])
        if not j:
            continue
        lst = out.setdefault(norm(en_alias(r["Card Name"])), [])
        if j not in lst:
            lst.append(j)
    return out


def base_name(n):
    b = re.sub(r"(ex|VSTAR|VMAX|V)$", "", n).strip()
    return b if b and b != n else None


# ---- fetching -------------------------------------------------------------

class Fetcher:
    def __init__(self, cache, refresh):
        self.cache, self.refresh = cache, refresh
        self.last = 0.0
        self.fails = 0
        os.makedirs(cache, exist_ok=True)

    def path(self, name, page):
        safe = name.replace("/", "_")
        return os.path.join(self.cache, safe + ("" if page == 1 else "_p%d" % page) + ".html")

    def get(self, name, page=1):
        p = self.path(name, page)
        if not self.refresh and os.path.exists(p):
            return open(p, encoding="utf-8").read()
        url = URL.format(w=urllib.parse.quote(name), p=page)
        for attempt in (1, 2):
            wait = 1.0 - (time.time() - self.last)
            if wait > 0:
                time.sleep(wait)
            self.last = time.time()
            try:
                req = urllib.request.Request(url, headers={"User-Agent": UA})
                with urllib.request.urlopen(req, timeout=30) as r:
                    body = r.read().decode("utf-8")
                self.fails = 0
                with open(p, "w", encoding="utf-8") as f:
                    f.write(body)
                return body
            except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError, OSError) as e:
                transient = not isinstance(e, urllib.error.HTTPError) or e.code >= 500
                if attempt == 1 and transient:
                    time.sleep(10)
                    continue
                self.fails += 1
                print("FAIL %s p%d: %s" % (name, page, e), file=sys.stderr)
                if self.fails >= 3:
                    raise SystemExit("3 consecutive failures; stopping")
                return None


# ---- parsing --------------------------------------------------------------

def clean(s):
    s = re.sub(r"<br\s*/?>", "\n", s)
    s = re.sub(r"<[^>]+>", "", s)
    s = html.unescape(s)
    return "\n".join(l.strip() for l in s.strip().splitlines()).strip()


def parse(body):
    """-> (hit count, [qa dict])"""
    m = re.search(r'<span class="HitNum">(\d+)</span>', body)
    hits = int(m.group(1)) if m else 0
    qas = []
    for item in re.split(r'<li class="FAQResultList_item">', body)[1:]:
        related = [{"name": clean(n), "id": int(i)} for i, n in
                   re.findall(r'<li class="CardName"><a href="[^"]*?/card/(\d+)/[^"]*"[^>]*>(.*?)</a>', item, re.S)]
        q = re.search(r'QuestionArea">\s*<div class="BodyArea">(.*?)</div>', item, re.S)
        a = re.search(r'AnswerArea">\s*<div class="BodyArea">(.*?)</div>', item, re.S)
        if q and a:
            qas.append({"related": related, "q": clean(q.group(1)), "a": clean(a.group(1))})
    return hits, qas


def search(fetch, name):
    """All Q&As for one search word, following pagination (10 per page)."""
    body = fetch.get(name, 1)
    if body is None:
        return None
    hits, qas = parse(body)
    pages = (hits + 9) // 10
    for p in range(2, pages + 1):
        b = fetch.get(name, p)
        if b is None:
            break
        qas += parse(b)[1]
    return hits, qas


def search_card(fetch, jp):
    r = search(fetch, jp)
    used = jp
    if r is not None and r[0] == 0:
        b = base_name(jp)
        if b:
            r2 = search(fetch, b)
            if r2 is not None:
                r, used = r2, b
    return used, (r[1] if r else [])


# ---- main -----------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry", nargs="+", help="search these Japanese names and print the parse")
    ap.add_argument("--refresh", action="store_true")
    ap.add_argument("--limit", type=int, help="only the first N distinct names")
    ap.add_argument("--csv-dir", default=DEF_CSV)
    ap.add_argument("--cache", default=DEF_CACHE)
    ap.add_argument("--out-dir", default=os.path.join(ROOT, "docs", "rulings"))
    a = ap.parse_args()
    fetch = Fetcher(a.cache, a.refresh)

    if a.dry:
        for n in a.dry:
            used, qas = search_card(fetch, n)
            print("== %s (searched %s): %d Q&As" % (n, used, len(qas)))
            print(json.dumps(qas, ensure_ascii=False, indent=1))
        return

    pool = json.load(open(os.path.join(ROOT, "data", "cards.json"), encoding="utf-8"))
    table = jp_names(a.csv_dir)
    key_jp, unmatched = {}, []
    for key, c in pool.items():
        js = table.get(norm(c["name"]))
        if js:
            key_jp[key] = js
        else:
            unmatched.append(key)
    distinct = []
    for js in key_jp.values():
        for j in js:
            if j not in distinct:
                distinct.append(j)
    if a.limit:
        distinct = distinct[:a.limit]
    print("pool %d, matched %d, distinct JP names %d, unmatched %d" %
          (len(pool), len(key_jp), len(distinct), len(unmatched)), file=sys.stderr)

    results = {}
    for i, j in enumerate(distinct, 1):
        results[j] = search_card(fetch, j)
        if i % 25 == 0:
            print("%d/%d" % (i, distinct and len(distinct)), file=sys.stderr)

    out, total_names_with = {}, 0
    for key, js in key_jp.items():
        seen, lst = set(), []
        for j in js:
            if j not in results:
                continue
            used, qas = results[j]
            for qa in qas:
                sig = (qa["q"], qa["a"])
                if sig in seen:
                    continue
                seen.add(sig)
                lst.append(dict(qa, search=used))
        if lst:
            out[key] = lst
    os.makedirs(a.out_dir, exist_ok=True)
    with open(os.path.join(a.out_dir, "jp-faq.json"), "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
    uniq = {(q["q"], q["a"]) for v in out.values() for q in v}
    today = datetime.date.today().isoformat()
    md = f"""# Official Japanese FAQ per pool card (jp-faq.json)

Source: https://www.pokemon-card.com/rules/faq/search.php?freeword=<URL-encoded Japanese name>&regulation_faq_main_item1=all
(follows `&page=N`, 10 Q&As per page). Fetched {today} by `tools/jp_faq.py`. Japanese names come from the
Kaggle playground EN/JP card lists (same Card ID = same printing); the full name is searched, and the
name without ex/V/VSTAR/VMAX only if the full name returned nothing. Free-word search also matches
card text, so a Q&A may be about another card that merely mentions this one (see `related`).
Not translated: read the Japanese.

Format: `{{ "<pool key>": [ {{"related": [{{"name", "id"}}], "q", "a", "search"}} ] }}`.
`id` is the pokemon-card.com card id (https://www.pokemon-card.com/card-search/details.php/card/<id>/regu/all).

Counts: pool cards {len(pool)}; matched to a Japanese name {len(key_jp)}; distinct Japanese names searched
{len(distinct)}; cards with Q&As {len(out)}; Q&A entries {sum(len(v) for v in out.values())}
({len(uniq)} distinct Q&As).

Unmatched pool cards (no Japanese name known; add by hand) [{len(unmatched)}]:
""" + "".join("- %s\n" % k for k in unmatched)
    with open(os.path.join(a.out_dir, "jp-faq.md"), "w", encoding="utf-8") as f:
        f.write(md)
    print(md)


if __name__ == "__main__":
    main()
