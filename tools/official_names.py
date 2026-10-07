"""Official English names for the Rust card database (names.rs / gen_carddb.py).

Rust's `CardDef.name`, `full_name`, `set`, `set_number` and attack / Ability
names are the official ones (data/official_text.json, data/pool.json); the
Twinleaf names live in the hidden `tl_*` fields and are used only at the oracle
boundary (canonical hash, descriptors, trace decoding).

Pool printings take their names from the official text of that printing.
A card outside the pool (older printing, support card) inherits the official
card / attack / Ability names of pool cards that share its Twinleaf card name
(so same-name rules behave exactly as before) and otherwise keeps Twinleaf's.
"""
import json, os, re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
COST = re.compile(r'^[GRWLPFDMCNY]+$|^0$')


def parse(text):
    """(card name, [attack names], [Ability names]) of an official card text."""
    lines = [l.strip() for l in text.split('\n') if l.strip()]
    atks, abil = [], []
    i = 1
    while i < len(lines):
        l = lines[i]
        if l == 'Ability:' and i + 1 < len(lines):
            abil.append(lines[i + 1])
            i += 2
            continue
        if COST.match(l) and i + 1 < len(lines):
            # "Name 30+" / "Name 20×" / "Name": strip a damage figure, never a trailing letter of the name
            m = re.match(r'^(.*?)(?:\s+(\d+)\s*[×x+\-]?)?$', lines[i + 1])
            atks.append(m.group(1).strip())
            i += 2
            continue
        i += 1
    return lines[0], atks, abil


class Names:
    def __init__(self, pool, cards):
        official = json.load(open(os.path.join(ROOT, 'data/official_text.json')))
        self.row = {r['fullName']: r for r in pool if r.get('fullName')}
        self.parsed = {}      # Twinleaf fullName -> (name, attacks, abilities)
        self.card = {}        # Twinleaf card name -> official card name
        self.atk = {}         # (Twinleaf card name, index-free key) -> official, by Twinleaf attack name
        self.pow = {}
        self.ambiguous = []
        for fn, r in self.row.items():
            c = cards[fn]
            name, a, b = parse(official['%s %s' % (r['set'], r['number'])]['text'])
            self.parsed[fn] = (name, a, b)
            self._put(self.card, c['name'], name, fn)
            for x, y in zip(c.get('attacks') or [], a):
                self._put(self.atk, (c['name'], x['name']), y, fn)
            for x, y in zip(c.get('powers') or [], b):
                self._put(self.pow, (c['name'], x['name']), y, fn)

    def _put(self, d, k, v, fn):
        if d.setdefault(k, v) != v:
            self.ambiguous.append((k, d[k], v, fn))

    def in_pool(self, c):
        return c['fullName'] in self.row

    def full_name(self, c):
        r = self.row.get(c['fullName'])
        return r['key'] if r else c['fullName']

    def card_name(self, c):
        p = self.parsed.get(c['fullName'])
        return p[0] if p else self.card.get(c['name'], c['name'])

    def set(self, c):
        r = self.row.get(c['fullName'])
        return r['set'] if r else c['set']

    def number(self, c):
        r = self.row.get(c['fullName'])
        return r['number'] if r else str(c.get('setNumber', ''))

    def attack(self, c, i, name):
        p = self.parsed.get(c['fullName'])
        if p:
            return p[1][i]
        return self.atk.get((c['name'], name), name)

    def power(self, c, i, name):
        p = self.parsed.get(c['fullName'])
        if p:
            return p[2][i]
        return self.pow.get((c['name'], name), name)

    def moves(self, c):
        """[(official, Twinleaf)] attack and Ability names of a Twinleaf card."""
        out = []
        for i, a in enumerate(c.get('attacks') or []):
            out.append((self.attack(c, i, a['name']), a['name']))
        for i, p in enumerate(c.get('powers') or []):
            out.append((self.power(c, i, p['name']), p['name']))
        return out

    def rename(self, name):
        """Official name for a Twinleaf card name (evolution links, stage tables)."""
        return self.card.get(name, name)


def build(pool, cards):
    n = Names(pool, cards)
    assert not n.ambiguous, n.ambiguous
    return n
