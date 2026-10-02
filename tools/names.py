"""English card names <-> Twinleaf full names.

The oracle and traces use Twinleaf's `fullName`, which mixes translated names
and print-variant suffixes ("Great Tree SCR", "Ceruledge exSAR PRE"). People
use the official English name with set and printing number: pool.json `key`
("Grand Tree SCR 136"), or "Name SET" when that is unique.
"""
import collections, json, os

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
