#!/usr/bin/env python3
r"""Events batch 9 census: card knowledge left in the engine core.

Three modes (one pass over the core's Rust files each; deterministic output, standard library only, `python3 -I` safe):

  --literals  every string literal in the core matched against every card name, attack name, Ability name, behavior
              class and key of data/cards.json and data/pool.json
  --idents    every identifier in the core matched against the CamelCase / snake_case forms of the same names, plus the
              card-data behavior flags and the dead Player / Rules fields the core still names
  --twinleaf  "Twinleaf" comment lines, JS-method names in backticks that are no Rust identifier, `TypeError` strings,
              `// Bn-OLD` markers, "as today", the retired engine's sources named in comments (.ts, oracle, JavaScript, V8)

The core is every .rs file under engine/src except cards/impls/, gen/ and bin/, minus `#[cfg(test)]` items and
`#[test]` functions. Run from anywhere: the repository root is found from this file's location (override with --root).

Limits of the test-code heuristic (token level, not a parser): an item behind `#[cfg(test)]` / `#[test]` (also
`cfg(all(test, ..))`; not `cfg(not(test))`) is skipped from the attribute to the end of its first `{ ... }` body or to its
first `;` at the same nesting level, whichever comes first; `#![cfg(test)]` skips the whole file. A `where` clause or a
const-generic `{N}` before the body would end the skip early (none exists in the core today); strings in macro bodies
count as code. Literals: ordinary, raw and byte strings are lexed (escapes decoded for \\ \" \' \n \t \r \0 and \u{..});
char literals and lifetimes are told apart by lookahead.

Matching:
  literals  exact (case-sensitive) equality with a name / attack / Ability / class / behavior / key / pool key, plus
            "contains" for multi-word names (a card or attack name of 2+ words inside a longer literal, on word
            boundaries, e.g. a message that embeds "Mist Energy"). Single-word names only match by equality.
  idents    the identifier is split into words (snake_case, SCREAMING_CASE, CamelCase); a name matches when its words
            (apostrophes dropped, "'s" kept as the letter or dropped: Boss's Orders = bossorders = bosssorders) are a
            contiguous run of the identifier's words. A single-word name matches only when it is the whole identifier
            (`Pikachu`, `DAMP`; written as a PascalCase or SCREAMING name: a snake_case word like `return` is never taken
            for a card; a rules word such as `Switch` is reported as a low-confidence match, see LOW_CONFIDENCE).
            Behavior classes match when the identifier equals the class.
"""
import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

# ----------------------------------------------------------------------------------------------------------------
# Lexer

IDENT_START = re.compile(r"[A-Za-z_]")
IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def lex(src):
    """Return a list of tokens (kind, text, line): kind in id, str, chr, life, lc (line comment), bc (block comment), p."""
    toks = []
    i = 0
    n = len(src)
    line = 1
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c in " \t\r":
            i += 1
            continue
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            toks.append(("lc", src[i:j], line))
            i = j
            continue
        if src.startswith("/*", i):
            depth = 0
            j = i
            start_line = line
            while j < n:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                else:
                    if src[j] == "\n":
                        line += 1
                    j += 1
            toks.append(("bc", src[i:j], start_line))
            i = j
            continue
        # raw strings r"..", r#".."#, br".."
        m = re.compile(r'b?r(#*)"').match(src, i)
        if m and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            hashes = m.group(1)
            end = '"' + hashes
            j = src.find(end, m.end())
            j = n if j < 0 else j
            body = src[m.end():j]
            toks.append(("str", body, line))
            line += src.count("\n", i, j + len(end))
            i = j + len(end)
            continue
        if c == '"' or (c == "b" and src.startswith('b"', i) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_"))):
            start_line = line
            j = i + (2 if c == "b" else 1)
            buf = []
            while j < n and src[j] != '"':
                ch = src[j]
                if ch == "\\":
                    nx = src[j + 1] if j + 1 < n else ""
                    if nx == "\n":
                        line += 1
                        j += 2
                        while j < n and src[j] in " \t\r\n":
                            if src[j] == "\n":
                                line += 1
                            j += 1
                        continue
                    if nx == "u" and src.startswith("{", j + 2):
                        k = src.find("}", j)
                        try:
                            buf.append(chr(int(src[j + 3:k].replace("_", ""), 16)))
                        except ValueError:
                            buf.append("?")
                        j = k + 1
                        continue
                    if nx == "x":
                        try:
                            buf.append(chr(int(src[j + 2:j + 4], 16)))
                        except ValueError:
                            buf.append("?")
                        j += 4
                        continue
                    buf.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(nx, nx))
                    j += 2
                    continue
                if ch == "\n":
                    line += 1
                buf.append(ch)
                j += 1
            toks.append(("str", "".join(buf), start_line))
            i = j + 1
            continue
        if c == "'":
            # char literal or lifetime
            m = re.compile(r"'(\\u\{[0-9a-fA-F_]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'])'").match(src, i)
            if m:
                toks.append(("chr", m.group(0), line))
                i = m.end()
                continue
            m = re.compile(r"'[A-Za-z_][A-Za-z0-9_]*").match(src, i)
            if m:
                toks.append(("life", m.group(0), line))
                i = m.end()
                continue
            i += 1
            continue
        if IDENT_START.match(c):
            m = IDENT_RE.match(src, i)
            toks.append(("id", m.group(0), line))
            i = m.end()
            continue
        toks.append(("p", c, line))
        i += 1
    return toks


def test_mask(toks, whole_file_attr):
    """Per token: True when inside a `#[cfg(test)]` item or a `#[test]` function."""
    n = len(toks)
    mask = [False] * n
    if whole_file_attr:
        return [True] * n
    i = 0
    pending = False
    while i < n:
        k, t, _ = toks[i]
        if k == "p" and t == "#" and i + 1 < n and toks[i + 1][1] == "[":
            # attribute: find the matching ]
            j = i + 2
            depth = 1
            while j < n and depth:
                if toks[j][1] == "[" and toks[j][0] == "p":
                    depth += 1
                elif toks[j][1] == "]" and toks[j][0] == "p":
                    depth -= 1
                j += 1
            words = [x[1] for x in toks[i + 2:j - 1] if x[0] == "id"]
            is_test = (words == ["test"]) or ("cfg" in words and "test" in words and "not" not in words)
            if is_test:
                pending = True
            if pending:
                for x in range(i, j):
                    mask[x] = True
            i = j
            continue
        if pending:
            # skip the item: to the first `{` body (brace matched) or the first `;` at depth 0
            j = i
            paren = 0
            while j < n:
                mask[j] = True
                kk, tt, _ = toks[j]
                if kk == "p":
                    if tt in "([":
                        paren += 1
                    elif tt in ")]":
                        paren -= 1
                    elif tt == ";" and paren <= 0:
                        j += 1
                        break
                    elif tt == "{" and paren <= 0:
                        depth = 1
                        j += 1
                        while j < n and depth:
                            mask[j] = True
                            if toks[j][0] == "p":
                                if toks[j][1] == "{":
                                    depth += 1
                                elif toks[j][1] == "}":
                                    depth -= 1
                            j += 1
                        break
                j += 1
            pending = False
            i = j
            continue
        i += 1
    return mask


# ----------------------------------------------------------------------------------------------------------------
# Files

def find_root(arg):
    if arg:
        return arg
    d = os.path.dirname(HERE)
    return d


def core_files(root, include_all=False):
    base = os.path.join(root, "engine", "src")
    out = []
    for dp, dn, fn in os.walk(base):
        dn.sort()
        rel = os.path.relpath(dp, base)
        parts = rel.split(os.sep)
        if not include_all and (parts[:2] == ["cards", "impls"] or parts[0] in ("gen", "bin")):
            continue
        for f in sorted(fn):
            if f.endswith(".rs"):
                out.append(os.path.join(dp, f))
    return out


def load_core(root):
    """[(relpath, tokens, mask)] for the core files."""
    res = []
    for path in core_files(root):
        src = open(path, encoding="utf-8").read()
        toks = lex(src)
        whole = bool(re.search(r"#!\[cfg\(test\)\]", src))
        mask = test_mask(toks, whole)
        res.append((os.path.relpath(path, root), toks, mask))
    return res


def all_rust_idents(root):
    s = set()
    for path in core_files(root, include_all=True):
        for k, t, _ in lex(open(path, encoding="utf-8").read()):
            if k == "id":
                s.add(t)
    return s


# ----------------------------------------------------------------------------------------------------------------
# Names

def load_names(root):
    cards = json.load(open(os.path.join(root, "data", "cards.json"), encoding="utf-8"))
    pool = json.load(open(os.path.join(root, "data", "pool.json"), encoding="utf-8"))
    kinds = {}  # string -> sorted set of kinds

    def add(s, kind):
        if s:
            kinds.setdefault(s.replace("\u2019", "'"), set()).add(kind)

    for s in EXTRA_NAMES:
        add(s, "card (not in data)")

    for key, c in cards.items():
        add(key, "key")
        add(c["name"], "card")
        add(c.get("class"), "class")
        add(c.get("behavior"), "behavior")
        for a in c.get("attacks", []):
            add(a["name"], "attack")
        for p in c.get("powers", []):
            add(p["name"], "ability")
    for p in pool:
        add(p["key"], "pool key")
        add(p["name"], "card")
    return kinds


def words_of(ident):
    """snake_case / SCREAMING_CASE / CamelCase to lower-case words ("HTTPServer2" -> http, server, 2)."""
    out = []
    for part in ident.split("_"):
        if not part:
            continue
        ws = re.findall(r"[A-Z]+(?![a-z])|[A-Z]?[a-z]+|[0-9]+", part)
        out.extend(w.lower() for w in ws)
    return out


def name_forms(name):
    """The word sequences a name can appear as in an identifier."""
    base = name.replace("’", "'")
    forms = set()
    for variant in (base, re.sub(r"'s\b", "", base), re.sub(r"'s\b", "s", base)):
        v = variant.replace("'", "")
        ws = [w.lower() for w in re.findall(r"[A-Za-z0-9À-ɏ]+", v)]
        if ws:
            forms.add(tuple(ws))
    return forms


def squash(ws):
    return "".join(ws)


# ----------------------------------------------------------------------------------------------------------------
# Allowlists

# Official card names the core is known to name that data/cards.json and data/pool.json don't contain (the deck rule's
# same-name aliases, plan 2.1: Sycamore / Juniper / Research are one name, Lysandre / Boss's Orders are one name).
EXTRA_NAMES = ["Professor Sycamore", "Professor Juniper", "Professor's Research", "Lysandre"]

# Literals that legitimately name a card (the rules text names it, or the trace boundary keys actions by name). Keyed by
# (file, literal): a literal matched in this file is reported as "allowed" with the reason. Nothing else is allowed.
ALLOWED_LITERALS = {
    # The printed tag vocabulary (`CardDef::tag_names`: the tags the card data prints, "Hop's", "Team Rocket's", "Unown"):
    # a table of printed words, not a card behavior; "Unown" is also a card name.
    ("engine/src/types.rs", "Unown"): "printed tag name table (types.rs TAG_NAMES)",
    # The trace boundary: the Twinleaf wire format names actions / prompts by card and attack name; C13 replaces them
    # with typed actions (batch-9-plan 2.1 "Legitimate").
    # (filled by the run; see the census-base.md table)
}

# Identifiers that match a name but are legitimate: the vocabulary the rules text itself needs (plan 2.1 "Legitimate").
# Keyed by identifier; every entry says why. `Pred::Name`, `Pred::NameContains`, `HasAbilityNamed`, `HasAttackNamed`,
# `SlotPred::Named`, `CausePred::Attack(name)` contain only the words "name(d)" / "attack", which are no card's name,
# so they never reach this list; entries here are matches that did.
ALLOWED_IDENTS = {}

# Single-word names that are also plain rules vocabulary: a match of the whole identifier is reported as LOW
# confidence (the identifier is almost surely the rules concept, not the card). Words in the card pool's names that are
# ordinary Rust / rules words. Printed here so the reviewer can audit each one.
LOW_CONFIDENCE = {
    "switch", "search", "retreat", "attack", "draw", "discard", "energy", "pokemon", "trainer", "item", "stadium",
    "supporter", "tool", "weakness", "resistance", "poison", "burn", "sleep", "paralysis", "confusion", "heal", "flip",
    "wait", "rest", "rage", "rock", "ghost", "fire", "water", "grass", "lightning", "psychic", "fighting", "darkness",
    "metal", "dragon", "fairy", "colorless", "basic", "stage", "attach", "evolve", "retaliate", "shuffle", "reveal",
    "return", "recover", "recycle", "lost", "prize", "bench", "active", "turn", "hand", "deck", "benched", "damage",
    "none", "any", "all", "other", "self", "scratch", "collect", "skip", "dig", "shred", "return",
}

# Readers of a card's name / key in the core: each site is a place where a card's identity can be compared.
NAME_READERS = {"full_name", "en_name", "en_key", "card_is", "def_by_full_name", "set_number"}

# Owner of each finding class: the batch 9 task that deletes it (see batch-9-plan 7). Matching is by identifier / literal
# (case-sensitive) or by the substring tag after "contains:". Unlisted findings print "?".
OWNER_LITERAL = {
    "Mist Energy": "C7 (trigger_order_matters)",
    "Hide 'n' Sneak": "C2 (if still present)",
    "Midnight Fluttering": "C2 (ActiveLock exempt name)",
    "Damp": "C1 (DAMP.exempt_name)",
    "Festival Grounds": "C3 (festival_lead_flag)",
    "Professor Sycamore": "C9 (deck_is_valid aliases)",
    "Professor Juniper": "C9 (deck_is_valid aliases)",
    "Professor's Research": "C9 (deck_is_valid aliases)",
    "Lysandre": "C9 (deck_is_valid aliases)",
    "Boss's Orders": "C9 (deck_is_valid aliases)",
}
OWNER_IDENT = {
    "MidnightFluttering": "C2", "Initialization": "C2", "CurlyWall": "C2", "CURLY_WALL": "C2", "RareCandy": "C2",
    "rare_candy_usable": "C2", "RareCandyUsable": "C2", "FestivalLead": "C3", "festival_lead_flag": "C3",
    "festival_lead": "C3", "legacy_energy_used": "C1", "legacy_energy": "C1",
    "ancient_pokemon_attacked_last_turn": "C1", "OtherAncientAttackedLastTurn": "C1",
    "NeutralCenter": "C8", "Retaliate": "C7 (AtkTrig retaliate)", "LittleGrudge": "NOT IN PLAN (card-named variant)",
    "little_grudge_discard": "NOT IN PLAN", "little_grudge_cont": "NOT IN PLAN", "HIDE_N_SNEAK": "NOT IN PLAN (const name; the Prevent declaration)",
    "DAMP": "C1", "Switch": "legitimate? (Op::Switch is the rules word)", "TRUMP_CARD_MARKER": "C6", "BOOMERANG_DISCARDED_MARKER": "C6",
}

# Card-data behavior flags and card-shaped state fields: the forms batch 9 deleted (a hit is a regression: the card
# knowledge went back into the core) and the registry names that remain. ident -> (where it lived, task that deleted it,
# note). Matching is on the identifier token; `class` / `behavior` only count as field reads (`.class`, `.behavior`).
FLAGS = {
    # CardDef / AttackDef / PowerDef flags deleted by C4 (the rules read declarations instead)
    "exempt_from_initialize": ("PowerDef", "C4", "deleted card-data flag"),
    "exempt_from_ability_lock": ("PowerDef", "C4", "deleted card-data flag"),
    "knocks_out_self": ("PowerDef", "C4", "deleted card-data flag (the program's consequence says it)"),
    "is_fossil": ("PowerDef", "C4", "deleted card-data flag"),
    "barrage": ("AttackDef / PowerDef", "C3/C4", "deleted card-data flag"),
    "use_on_bench": ("AttackDef", "C4", "deleted card-data flag"),
    "gx_attack": ("AttackDef", "C4", "deleted card-data flag"),
    "shred_attack": ("AttackDef", "C4", "deleted card-data flag"),
    "has_effect_fn": ("AttackDef / PowerDef", "C4", "deleted card-data flag"),
    "use_when_in_play": ("PowerDef", "C4", "deleted card-data flag"),
    "use_from_hand": ("PowerDef", "C4", "deleted card-data flag"),
    "use_from_hand_to_bench": ("PowerDef", "C4", "deleted card-data flag"),
    "use_from_discard": ("PowerDef", "C4", "deleted card-data flag"),
    "copycat_attack": ("AttackDef", "C4", "deleted card-data flag"),
    "can_use_on_first_turn": ("AttackDef", "C4", "deleted card-data flag"),
    "attaches_to_opponents_pokemon": ("CardDef", "C4", "deleted card-data flag"),
    "fossil_doll": ("CardDef", "C4", "deleted card-data flag"),
    "methods": ("CardDef", "C4", "deleted class method list"),
    "has_method": ("CardDef", "C4", "deleted reader of the method list"),
    # the behavior class names that remain: the registry key of a card's port (card knowledge by design)
    "class": ("CardDef", "kept", "the behavior class name (registry key, read as `.class`)"),
    "behavior": ("CardDef", "kept", "the behavior class name (registry key, read as `.behavior`)"),
    # Player / Slot / CardInst / Rules fields deleted by C1 / C3 / C9
    "used_dragons_wish": ("Player", "C1", "deleted dead field"),
    "unlimited_energy_attach_turns_remaining": ("Player", "C1", "deleted dead field"),
    "stadium_and_tool_have_no_effect_turns_remaining": ("Player", "C1", "deleted dead field"),
    "can_evolve": ("Player", "C1", "deleted dead field"),
    "unlimited_energy_attachments": ("Rules", "C1", "deleted dead field"),
    "ancient_pokemon_attacked_last_turn": ("Player", "C1", "deleted card-shaped record"),
    "legacy_energy_used": ("Player", "C1", "deleted card-shaped record"),
    "attack_barrage": ("CardInst", "C3", "deleted Festival Lead flag on the card"),
    "barrage_used": ("Effect::UseAttack / Attack", "C3", "deleted"),
    "write_barrage": ("fn", "C3", "deleted Festival Lead flag writer"),
    "player_id": ("fn", "C9", "deleted: a player is the player index (the trace boundary writes index + 1)"),
    "player_index_by_id": ("fn", "C9", "deleted: a player is the player index"),
    "perspective_id": ("fn", "C9", "deleted: a prompt names its player by index"),
    "intern": ("markers.rs", "C6", "deleted string-keyed markers"),
    "marker_id": ("markers.rs", "C6", "deleted string-keyed markers"),
}
# `.field` reads whose bare word is common: counted only after a dot.
FLAGS_DOT_ONLY = {"class": FLAGS["class"], "behavior": FLAGS["behavior"], "methods": FLAGS["methods"]}
# Identifiers of the deleted marker machinery (C1 / C6): any hit is a regression.
MARKER_IDENTS = {"SetMarker": "C6", "ClearMarker": "C6", "HasMarker": "C6", "PlayerMarker": "C6",
                 "DAMAGE_DEALT_MARKER": "C1", "KNOCKOUT_MARKER": "C1", "CLEAR_KNOCKOUT_MARKER": "C1",
                 "LOST_CITY_MARKER": "C1", "COIN_REFLIP_AGAIN_USED": "C6", "EXTRA_LEN": "C6",
                 "TRUMP_CARD_MARKER": "C6", "BOOMERANG_DISCARDED_MARKER": "C6"}


# ----------------------------------------------------------------------------------------------------------------
# Modes

def mode_literals(root, files):
    kinds = load_names(root)
    multi = [(s, "/".join(sorted(k))) for s, k in kinds.items() if len(s.split()) >= 2 and "key" not in k and "pool key" not in k
             and "class" not in k and "behavior" not in k]
    rows = []
    for rel, toks, mask in files:
        for (k, t, ln), m in zip(toks, mask):
            if m or k != "str" or not t:
                continue
            t = t.replace("\u2019", "'")
            hit = None
            if t in kinds:
                hit = "equals " + "/".join(sorted(kinds[t]))
            else:
                for s, kd in multi:
                    if len(s) >= 6 and s in t and re.search(r"(?<![A-Za-z0-9])" + re.escape(s) + r"(?![A-Za-z0-9])", t):
                        hit = "contains %s '%s'" % (kd, s)
                        break
            if hit:
                tag = "allowed: " + ALLOWED_LITERALS[(rel, t)] if (rel, t) in ALLOWED_LITERALS else "OPEN"
                owner = next((v for kk, v in OWNER_LITERAL.items() if kk in t), "?")
                rows.append((rel, ln, t, hit, tag, owner))
    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    for rel, ln, t, hit, tag, owner in rows:
        print("%s:%d\t%s\t%s\t%s\towner %s" % (rel, ln, json.dumps(t, ensure_ascii=False), hit, tag, owner))
    print("# literals matched: %d" % len(rows))
    print()
    print("## sites that compare or look up a card by name (identifier tokens; a card name reaches the core as data)")
    n = 0
    for rel, toks, mask in files:
        for idx, ((k, t, ln), m) in enumerate(zip(toks, mask)):
            if m or k != "id" or t not in NAME_READERS:
                continue
            print("%s:%d\t%s" % (rel, ln, t))
            n += 1
    print("# name-reader sites: %d" % n)
    return len(rows)


def mode_idents(root, files):
    kinds = load_names(root)
    # name forms -> (kind, original)
    multi = {}
    single = {}
    classes = {}
    for s, k in kinds.items():
        if "key" in k or "pool key" in k:
            continue
        if "class" in k or "behavior" in k:
            classes[s] = "/".join(sorted(k))
            continue
        forms = name_forms(s)
        for f in forms:
            (multi if len(f) >= 2 else single).setdefault(f, set()).add((s, "/".join(sorted(k))))
    # index multi-word forms by their first word for speed
    by_first = {}
    for f, v in multi.items():
        by_first.setdefault(f[0], []).append((f, v))
    rows = []
    flag_rows = []
    marker_rows = []
    seen = set()
    for rel, toks, mask in files:
        for idx, ((k, t, ln), m) in enumerate(zip(toks, mask)):
            if m or k != "id":
                continue
            prev = toks[idx - 1][1] if idx else ""
            prev_kind = toks[idx - 1][0] if idx else ""
            # flags
            if t in FLAGS and not (t in ("class", "behavior", "methods") and prev != "."):
                flag_rows.append((rel, ln, t, FLAGS[t]))
            elif t in FLAGS_DOT_ONLY and prev == "." and prev_kind == "p":
                flag_rows.append((rel, ln, "." + t, FLAGS_DOT_ONLY[t]))
            if t in MARKER_IDENTS:
                marker_rows.append((rel, ln, t, MARKER_IDENTS[t]))
            key = (rel, ln, t)
            if key in seen:
                continue
            seen.add(key)
            ws = words_of(t)
            if not ws:
                continue
            found = None
            # whole-identifier class match
            if t in classes:
                low = "low-confidence " if t.lower() in LOW_CONFIDENCE else ""
                found = (low + "class " + classes[t], "?")
            if not found:
                for i in range(len(ws)):
                    for f, v in by_first.get(ws[i], ()):
                        L = len(f)
                        if tuple(ws[i:i + L]) == f:
                            s, kd = sorted(v)[0]
                            found = ("%s '%s'" % (kd, s), "?")
                            break
                    if found:
                        break
            if not found:
                # a single-word name matches only as the whole identifier written as that name: `Pikachu`, `Retaliate`
                # (PascalCase), `DAMP` (SCREAMING); a snake_case word (`return`, `collect`) is never taken for a card
                for f, v in single.items():
                    w = f[0]
                    if len(ws) == 1 and ws[0] == w and ((t[0].isupper() and "_" not in t) or (t.isupper() and len(t) > 1)):
                        s, kd = sorted(v)[0]
                        low = "low-confidence " if w in LOW_CONFIDENCE else ""
                        found = ("%sequals single-word %s '%s'" % (low, kd, s), "?")
                        break
            if found:
                if t in ALLOWED_IDENTS:
                    tag = "allowed: " + ALLOWED_IDENTS[t]
                elif found[0].startswith("low-confidence"):
                    tag = "low"
                else:
                    tag = "OPEN"
                owner = OWNER_IDENT.get(t, "?")
                rows.append((rel, ln, t, found[0], tag, owner))
    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    print("## identifiers matching a card / attack / Ability / class name")
    for rel, ln, t, hit, tag, owner in rows:
        print("%s:%d\t%s\t%s\t%s\towner %s" % (rel, ln, t, hit, tag, owner))
    print("# identifier matches: %d (OPEN %d, low-confidence %d)" % (len(rows), sum(1 for r in rows if r[4] == "OPEN"),
                                                                    sum(1 for r in rows if r[4] == "low")))
    print()
    print("## behavior flags and card-shaped fields (identifier tokens; one line per use; every one but class / behavior is a regression)")
    flag_rows.sort(key=lambda r: (r[2], r[0], r[1]))
    for rel, ln, t, (where, owner, note) in flag_rows:
        print("%s:%d\t%s\t%s\towner %s\t%s" % (rel, ln, t, where, owner, note))
    by = {}
    for r in flag_rows:
        by[r[2]] = by.get(r[2], 0) + 1
    print("# flag uses by name: " + ", ".join("%s=%d" % kv for kv in sorted(by.items())))
    print()
    print("## deleted marker machinery (identifier tokens; every hit is a regression)")
    marker_rows.sort(key=lambda r: (r[2], r[0], r[1]))
    mc = {}
    for rel, ln, t, owner in marker_rows:
        mc[t] = mc.get(t, 0) + 1
    print("# marker identifier uses by name: " + ", ".join("%s=%d" % kv for kv in sorted(mc.items())))
    for rel, ln, t, owner in marker_rows:
        print("%s:%d\t%s\towner %s" % (rel, ln, t, owner))
    return len(rows)


BT_RE = re.compile(r"`([^`\n]+)`")
OLD_RE = re.compile(r"\bB\d+[a-z]?-OLD\b")
# The retired engine's sources named in a comment: TypeScript file names, the oracle, JavaScript, V8.
SOURCE_RE = re.compile(r"\b[a-z][a-z0-9-]*\.ts\b|\boracle\b|\bJavaScript\b|\bV8\b", re.I)


def mode_twinleaf(root, files):
    rust = all_rust_idents(root)
    out = {"twinleaf": [], "js": [], "typeerror": [], "old": [], "today": [], "source": []}
    for rel, toks, mask in files:
        for (k, t, ln), m in zip(toks, mask):
            if m:
                continue
            if k in ("lc", "bc"):
                # a block comment's lines count separately
                lines = t.split("\n")
                for off, text in enumerate(lines):
                    l = ln + off
                    if "Twinleaf" in text:
                        out["twinleaf"].append((rel, l, text.strip()))
                    if "TypeError" in text:
                        out["typeerror"].append((rel, l, "(comment) " + text.strip()))
                    if SOURCE_RE.search(text):
                        out["source"].append((rel, l, text.strip()))
                    if OLD_RE.search(text):
                        out["old"].append((rel, l, text.strip()))
                    if re.search(r"\bas today\b", text, re.I):
                        out["today"].append((rel, l, text.strip()))
                    for span in BT_RE.findall(text):
                        if "@" in span:
                            continue  # a registry key (`Class@SET`), not a method name
                        bad = []
                        for w in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", span):
                            if re.search(r"[a-z][A-Z]", w) and "_" not in w and w not in rust:
                                bad.append(w)
                        if bad:
                            out["js"].append((rel, l, "`%s` -> %s" % (span, ",".join(sorted(set(bad))))))
            elif k == "str":
                if "TypeError" in t:
                    out["typeerror"].append((rel, ln, json.dumps(t, ensure_ascii=False)))
                if "Twinleaf" in t:
                    out["twinleaf"].append((rel, ln, "(string) " + json.dumps(t, ensure_ascii=False)))
                if re.search(r"\bas today\b", t, re.I):
                    out["today"].append((rel, ln, "(string) " + json.dumps(t, ensure_ascii=False)))
    titles = [("twinleaf", "\"Twinleaf\" in comments / strings"), ("js", "backticked JS-method names that are no Rust identifier"),
              ("typeerror", "TypeError strings and comments"), ("old", "// Bn-OLD markers"), ("today", "\"as today\""),
              ("source", "the retired engine's sources in comments (.ts file names, oracle, JavaScript, V8)")]
    for key, title in titles:
        rows = sorted(set(out[key]))
        print("## %s: %d lines in %d files" % (title, len(rows), len(set(r[0] for r in rows))))
        for rel, ln, text in rows:
            print("%s:%d\t%s" % (rel, ln, text))
        print()
    return sum(len(set(v)) for v in out.values())


def card_files(root):
    base = os.path.join(root, "engine", "src", "cards", "impls")
    return [os.path.join(base, f) for f in sorted(os.listdir(base)) if f.endswith(".rs")]


def mode_markers(root):
    """Usage-limit census over the card files (cards/impls): who declares which usage limit (`Once::PerTurn`,
    `Once::PerTurnShared`, a `once:` field), and the EndTurn triggers with what their programs do. A name is a literal or a
    `const NAME: &str = ".."` of the same file; an identifier imported from elsewhere prints as <IDENT>. (The string-keyed
    marker forms C6 deleted are reported by `--idents` as regressions.)"""
    files = {}
    names = {"PerTurn": set(), "PerTurnShared": set(), "once:": set()}
    endturn = []
    for path in card_files(root):
        rel = os.path.basename(path)
        toks = lex(open(path, encoding="utf-8").read())
        mask = test_mask(toks, False)
        toks = [t for t, m in zip(toks, mask) if not m]
        n = len(toks)
        consts = {}
        for i, (k, t, ln) in enumerate(toks):
            if k == "id" and t in ("const", "static") and i + 1 < n and toks[i + 1][0] == "id":
                for j in range(i + 2, min(i + 10, n)):
                    if toks[j][0] == "str":
                        consts[toks[i + 1][1]] = toks[j][1]
                        break
                    if toks[j][1] == ";":
                        break

        def resolve(j):
            if j >= n:
                return "?"
            k, t, _ = toks[j]
            if k == "str":
                return t
            if k == "id":
                return consts.get(t, "<" + t + ">")
            return "?"

        for i, (k, t, ln) in enumerate(toks):
            if k != "id":
                continue
            if t in ("PerTurn", "PerTurnShared") and i >= 3 and toks[i - 3][1] == "Once":
                nm = resolve(i + 2)
                files.setdefault(rel, []).append((ln, "Once::" + t, nm))
                names[t].add(nm)
            elif t == "once" and i + 2 < n and toks[i + 1][1] == ":" and toks[i + 2][0] == "id" and toks[i + 2][1] != "Once":
                nm = resolve(i + 2)
                files.setdefault(rel, []).append((ln, "once:", nm))
                names["once:"].add(nm)
            elif t == "Trigger" and i + 1 < n and toks[i + 1][1] == "{":
                depth = 0
                j = i + 1
                body = []
                while j < n:
                    if toks[j][0] == "p" and toks[j][1] == "{":
                        depth += 1
                    elif toks[j][0] == "p" and toks[j][1] == "}":
                        depth -= 1
                        if depth == 0:
                            break
                    body.append(toks[j])
                    j += 1
                m = len(body)
                is_end = any(body[x][1] == "EventKind" and x + 3 < m and body[x + 3][1] == "EndTurn" for x in range(m))
                if is_end:
                    ops = sorted({body[x + 3][1] for x in range(m - 3) if body[x][1] == "Op" and body[x + 1][1] == ":"})
                    via_const = (not ops) and any(b[1] == "steps" for b in body)
                    endturn.append((rel, ln, ops, via_const))
    print("## card files declaring a usage limit (Once::PerTurn(Shared) / once:): %d" % len(files))
    for rel in sorted(files):
        print("%s: %s" % (rel, "; ".join("%d %s %s" % (ln, op, nm) for ln, op, nm in files[rel])))
    print()
    for kk in sorted(names):
        print("# distinct names for %-14s %d: %s" % (kk, len(names[kk]), ", ".join(sorted(str(x) for x in names[kk]))))
    allp = names["PerTurn"] | names["PerTurnShared"]
    print("# distinct PerTurn + PerTurnShared names: %d" % len(allp))
    print()
    print("## triggers whose event is EventKind::EndTurn (the card's `Trigger { .. }` blocks)")
    files_end = set()
    for rel, ln, ops, via_const in endturn:
        files_end.add(rel)
        flag = "(steps via a const: read the file)" if via_const else "ops"
        print("%s:%d\t%s\t%s" % (rel, ln, ",".join(ops) or "-", flag))
    print("# files with an EndTurn trigger: %d" % len(files_end))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--literals", action="store_true")
    ap.add_argument("--idents", action="store_true")
    ap.add_argument("--twinleaf", action="store_true")
    ap.add_argument("--markers", action="store_true", help="usage-limit and EndTurn-trigger census over the card files")
    ap.add_argument("--root", default=None, help="repository root (default: the checkout this script is in)")
    a = ap.parse_args()
    if not (a.literals or a.idents or a.twinleaf or a.markers):
        ap.error("one of --literals / --idents / --twinleaf / --markers")
    root = find_root(a.root)
    if a.markers:
        mode_markers(root)
        if not (a.literals or a.idents or a.twinleaf):
            return 0
    files = load_core(root)
    print("# core files: %d (engine/src minus cards/impls, gen, bin; test code skipped)" % len(files))
    if a.literals:
        mode_literals(root, files)
    if a.idents:
        mode_idents(root, files)
    if a.twinleaf:
        mode_twinleaf(root, files)


if __name__ == "__main__":
    sys.exit(main())
