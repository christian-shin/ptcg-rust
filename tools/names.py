"""English card, attack and Ability names <-> Twinleaf names.

The oracle and traces use Twinleaf's `fullName`, which mixes translated names
and print-variant suffixes ("Great Tree SCR", "Ceruledge exSAR PRE"). People
use the official English name with set and printing number: pool.json `key`
("Grand Tree SCR 136"), or "Name SET" when that is unique. The Rust engine
itself speaks official names (CardDef.name / full_name, attack and Ability
names) and keeps Twinleaf's as hidden `tl_*` fields; anything sent to the
oracle (scenario edits, scripted answers) goes through `twinleaf()` and
`move_twinleaf()`.
"""
import collections, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
_pool = [r for r in json.load(open(os.path.join(ROOT, 'data/pool.json'))) if r.get('fullName')]
_to_tl = {}
_short = collections.defaultdict(set)
for r in _pool:
    _to_tl[r['key']] = r['fullName']
    _short[r['key'].rsplit(' ', 1)[0]].add(r['fullName'])
for s, fns in _short.items():
    if len(fns) == 1:
        _to_tl.setdefault(s, next(iter(fns)))
_to_en = {r['fullName']: r['key'] for r in _pool}


def twinleaf(name):
    """Twinleaf full name for an English key / "Name SET" (Twinleaf names pass through)."""
    return _to_tl.get(name, name)


def english(full_name):
    """English key for a Twinleaf full name (unchanged if not a pool card)."""
    return _to_en.get(full_name, full_name)


def label(name):
    """Human-readable card label: the English key, with the Twinleaf full name
    in parentheses when it differs ("Growing Grass Energy POR 86 (Grow [G] Energy M3)")."""
    tl = twinleaf(name)
    en = english(tl)
    return en if en == tl else '%s (%s)' % (en, tl)


_moves = None


def _move_table():
    """Twinleaf attack / Ability name -> [(official, [Twinleaf card full names])]."""
    global _moves
    if _moves is None:
        import official_names
        cards = {c['fullName']: c for c in json.load(open(os.path.join(ROOT, 'data/twinleaf-cards.json')))}
        on = official_names.build(json.load(open(os.path.join(ROOT, 'data/pool.json'))), cards)
        _moves = collections.defaultdict(lambda: collections.defaultdict(set))
        for fn, c in cards.items():
            for off, tl in on.moves(c):
                _moves[off][tl].add(fn)
    return _moves


def move_twinleaf(name, card_names=()):
    """Twinleaf name of an attack / Ability given by its official name (Twinleaf names pass through).

    Two Twinleaf cards can use the same official name for different Twinleaf names
    ("Spooky Shot" is Haunter ASC's "Hollow Shot"); `card_names` (the scenario's
    cards, English keys or Twinleaf full names) picks the one that is in play.
    """
    table = _move_table().get(name)
    if not table:
        return name
    have = {twinleaf(c) for c in card_names}
    mine = {tl for tl, fns in table.items() if fns & have}
    if len(mine) == 1:
        return next(iter(mine))
    if len(mine) > 1:
        return name if name in mine else sorted(mine)[0]
    if name in table or len(table) != 1:
        return name
    return next(iter(table))
