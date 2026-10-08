"""Game session for the web client: wraps `ptcg.Env`, hides what player 0 may
not see, resolves bot turns, builds card data and a game log."""
import glob
import json
import os
import random
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "python"))
sys.path.insert(0, os.path.join(ROOT, "tools"))

import ptcg  # noqa: E402

from policies import make_policy  # noqa: E402

ME, OPP = 0, 1

TYPE_LETTER = {1: "G", 2: "R", 3: "W", 4: "L", 5: "P", 6: "F", 7: "D", 8: "M", 9: "C", 10: "Y", 11: "N"}
TYPE_NAME = {1: "Grass", 2: "Fire", 3: "Water", 4: "Lightning", 5: "Psychic", 6: "Fighting",
             7: "Darkness", 8: "Metal", 9: "Colorless", 10: "Fairy", 11: "Dragon"}
STAGE = {2: "Basic", 3: "Stage 1", 4: "Stage 2", 5: "VMAX", 6: "VSTAR", 7: "V-UNION", 8: "LEGEND",
         9: "Mega", 10: "BREAK", 11: "Lv.X", 1: "Restored"}
TRAINER = {0: "Item", 1: "Supporter", 2: "Stadium", 3: "Tool"}
CONDITION = {0: "Paralyzed", 1: "Confused", 2: "Asleep", 3: "Poisoned", 4: "Burned"}
SELECT_TYPE = {0: "main", 1: "card", 2: "attachedCard", 3: "cardOrAttachedCard", 4: "energy", 5: "skill",
               6: "attack", 7: "evolve", 8: "count", 9: "yesNo", 10: "specialCondition"}
CONTEXT = {0: "Main", 1: "Choose your Active Pokémon", 2: "Choose Benched Pokémon", 3: "Switch", 4: "Move to Active",
           5: "Put on Bench", 6: "Put into play", 7: "Take into hand", 8: "Discard", 9: "Put on deck",
           10: "Put on bottom of deck", 11: "Prize", 12: "Do not move", 13: "Damage counters",
           22: "Attach to", 24: "Look", 25: "Choose target", 26: "Discard attached card",
           28: "Move attached card", 30: "Discard Energy", 35: "Attack", 38: "Draw count",
           41: "Go first?", 42: "Mulligan", 43: "Use ability?", 46: "Coin flip", 49: "Shuffle"}
AREA = {1: "deck", 2: "hand", 3: "discard", 4: "active", 5: "bench", 6: "prize", 7: "stadium", 8: "energy",
        9: "tool", 10: "pre-evolution", 11: "player", 12: "revealed"}
# Short prompt titles (the binding gives only a numeric context).
SHORT_CONTEXT = {1: "Active", 2: "Bench", 3: "Switch", 4: "To Active", 5: "To Bench", 6: "Into play",
                 7: "To hand", 8: "Discard", 9: "To deck", 10: "Deck bottom", 11: "Prize", 13: "Damage counters",
                 22: "Attach", 24: "Look", 25: "Target", 26: "Discard attached", 28: "Move attached",
                 30: "Discard Energy", 35: "Attack", 38: "Draw", 41: "Go first?", 42: "Mulligan",
                 43: "Use ability?", 46: "Coin flip", 49: "Shuffle"}
KIND = {0: "number", 1: "yes", 2: "no", 3: "card", 4: "toolCard", 5: "energyCard", 6: "energy", 7: "play",
        8: "attach", 9: "evolve", 10: "ability", 11: "discard", 12: "retreat", 13: "attack", 14: "end",
        15: "skill", 16: "specialCondition"}


def _load(path):
    with open(os.path.join(ROOT, path), encoding="utf-8") as f:
        return json.load(f)


class CardDB:
    """Printed card data (data/cards.json) by engine ref ("SET-NUMBER")."""

    def __init__(self):
        self.by_full = _load("data/cards.json")   # card key -> printed data
        self.by_setnum = {}
        for c in self.by_full.values():
            self.by_setnum.setdefault("%s-%s" % (c["set"], c["setNumber"]), c)
        self.cache = {}

    def card(self, ref):
        c = self.cache.get(ref)
        if c is None:
            c = self.cache[ref] = self._build(ref)
        return c

    def raw(self, ref):
        return self.by_setnum.get(ref)

    def _build(self, ref):
        raw = self.raw(ref)
        if raw is None:
            return {"ref": ref, "name": ref, "super": "unknown"}
        name, set_, number = raw["name"], raw["set"], raw["setNumber"]
        sup = {1: "pokemon", 2: "trainer", 3: "energy"}.get(raw["superType"], "unknown")
        d = {"ref": ref, "name": name, "set": set_, "number": number, "super": sup}
        if set_ and number:
            num = number.zfill(3) if number.isdigit() else number
            d["image"] = "https://limitlesstcg.nyc3.cdn.digitaloceanspaces.com/tpci/%s/%s_%s_R_EN_%%s.png" % (set_, set_, num)
        if sup == "pokemon":
            d["stage"] = STAGE.get(raw.get("stage"), "")
            d["types"] = [TYPE_LETTER.get(t, "?") for t in raw.get("cardType", [])]
            d["hp"] = raw.get("hp", 0)
            d["evolvesFrom"] = raw.get("evolvesFrom", "")
            d["retreat"] = [TYPE_LETTER.get(t, "?") for t in raw.get("retreat", [])]
            d["weakness"] = [TYPE_LETTER.get(w["type"], "?") + ("x2" if not w.get("value") else "") for w in raw.get("weakness", [])]
            d["resistance"] = [TYPE_LETTER.get(w["type"], "?") + str(w.get("value", -30)) for w in raw.get("resistance", [])]
            d["attacks"] = [{"name": a["name"], "cost": [TYPE_LETTER.get(t, "?") for t in a.get("cost", [])],
                             "damage": a.get("damage", 0), "calc": a.get("damageCalculation", ""),
                             "text": a.get("text", "")} for a in raw.get("attacks", [])]
            d["powers"] = [{"name": p["name"], "text": p.get("text", ""), "kind": p.get("powerType")} for p in raw.get("powers", [])]
            if "ex" in raw.get("cardTag", []) or name.endswith(" ex"):
                d["ex"] = True
        elif sup == "trainer":
            d["trainerType"] = TRAINER.get(raw.get("trainerType"), "Trainer")
            d["text"] = raw.get("text", "")
        elif sup == "energy":
            d["energyType"] = "Special" if raw.get("energyType") == 1 else "Basic"
            d["provides"] = [TYPE_LETTER.get(t, "?") for t in raw.get("provides", [])]
            d["text"] = raw.get("text", "")
        return d


def _pretty(slug):
    t = slug.replace("-", " ").title().replace(" S ", "'s ")
    return " ".join("ex" if w == "Ex" else w for w in t.split(" "))


def _playable():
    """The playable meta decks (tools/meta_decks.py): {id: [card key per copy]}."""
    return {d["name"]: d["cards"] for d in _load("decks/meta/playable.corpus.json")["decks"]}


def list_decks():
    return [{"id": i, "name": _pretty(i), "count": len(cards)} for i, cards in sorted(_playable().items())]


def read_deck(deck_id):
    cards = _playable().get(deck_id)
    if cards is None:
        raise ValueError("bad deck id")
    return list(cards)


def deck_listing(deck_id, db=None):
    """[{count, name, card?}] for a deck, for display (card = printed data incl. image)."""
    cards = {}
    for n in read_deck(deck_id):
        cards[n] = cards.get(n, 0) + 1
    out = []
    for n, c in cards.items():
        row = {"count": c, "name": n}
        raw = db.by_full.get(n) if db else None
        if raw:
            row["card"] = db.card("%s-%s" % (raw["set"], raw["setNumber"]))
            row["name"] = row["card"]["name"]
        out.append(row)
    return out


def ref_of(cid):
    return cid.split("#")[0]


def serial_of(cid):
    return int(cid.split("#")[1])


class GameSession:
    def __init__(self, deck_a, deck_b, seed=None, bot="random", cards=None):
        self.cards = cards or CardDB()
        self.seed = random.getrandbits(31) if seed is None else int(seed)
        self.deck_ids = (deck_a, deck_b)
        self.env = ptcg.Env(read_deck(deck_a), read_deck(deck_b), self.seed)
        self.bot = make_policy(bot, self.seed ^ 0x5EED)
        self.log = []
        self.error = None
        self.stuck = False
        self._refresh()
        self.prev = self._snapshot()
        self._just_played = set()
        self._last_kind = None
        self.say("sys", "Seed %d · %s bot" % (self.seed, bot), "info")
        self._run_bot()

    # ---- engine plumbing -------------------------------------------------
    def _refresh(self):
        self.sel = self.env.select()
        self.state = json.loads(self.env.canonical())
        # Serial -> card id. Cards in temporary lists (looked-at deck tops, revealed
        # cards) are missing from canonical(), so keep every serial ever seen: all
        # cards start in a deck or hand, and serials never change.
        if not hasattr(self, "serials"):
            self.serials = {}
        for p in self.state["players"]:
            for zone in ("deck", "hand", "discard", "lostzone", "stadium", "supporter"):
                for c in p[zone]:
                    self.serials[serial_of(c)] = c
            for pr in p["prizes"]:
                for c in pr:
                    self.serials[serial_of(c)] = c
            for s in [p["active"]] + p["bench"]:
                for c in s["cards"]:
                    self.serials[serial_of(c)] = c

    @property
    def over(self):
        return self.sel is None

    @staticmethod
    def allows_repeats(sel):
        return sel["context"] == 13

    def try_answer(self, indices):
        if self.sel is None:
            return False
        c = self.env.clone()
        try:
            c.step(list(indices))
        except BaseException as e:  # noqa: BLE001 (engine panics are BaseException)
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            return False
        return not self._dead_end(c)

    @staticmethod
    def _dead_end(env):
        """True when `env` waits on a decision nobody can legally answer (a card
        with no legal choice, such as Glass Trumpet with no Benched Colorless Pokémon)."""
        if env.done or env.select() is None:
            return False
        probe = env.clone()
        try:
            return not probe.step_random(random.getrandbits(32))
        except BaseException as e:  # noqa: BLE001
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            return True

    def validate(self, indices):
        """None when `indices` is a legal answer to the current decision, else the engine's error."""
        if self.sel is None:
            return "game over"
        c = self.env.clone()
        try:
            c.step(list(indices))
        except BaseException as e:  # noqa: BLE001
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            return str(e) or type(e).__name__
        return "dead end: no legal follow-up" if self._dead_end(c) else None

    def _apply(self, indices, who):
        """Apply an answer for `who`, log it, then log the visible consequences."""
        sel, before = self.sel, self.state
        text = self._describe_answer(sel, indices, who)
        backup = self.env.clone()
        try:
            if indices is None:
                ok = self.env.step_random(random.getrandbits(32))
                if not ok:
                    raise ValueError("no legal answer found")
            else:
                self.env.step(list(indices))
        except BaseException as e:  # noqa: BLE001
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            self.env = backup
            raise
        if text:
            self.say("you" if who == ME else "opp", text)
        self._refresh()
        self._diff_log()

    def _run_bot(self):
        guard = 0
        while self.sel is not None and guard < 20000:
            guard += 1
            sel = self.sel
            if sel["player"] == ME:
                if sel["option"] or sel["maxCount"] > 0:
                    return
                # Nothing to choose: acknowledge automatically.
                try:
                    self._apply([], ME)
                except Exception as e:  # noqa: BLE001
                    self._fail("auto-continue rejected: %s" % e)
                    return
                continue
            try:
                ans = self.bot.choose(self)
                if ans is not None and not self.try_answer(ans):
                    # Rejected when applied, or leads to a dead end: take the first single
                    # pick that works, else let the engine pick.
                    n = len(sel["option"])
                    cands = [[i] for i in random.sample(range(n), n)] + ([[]] if sel["minCount"] == 0 else [])
                    ans = next((c for c in cands if self.try_answer(c)), None)
                self._apply(ans, OPP)
            except BaseException as e:  # noqa: BLE001
                if isinstance(e, (KeyboardInterrupt, SystemExit)):
                    raise
                self._fail("bot failed: %s" % e)
                return
        if self.sel is None:
            self._finish()
        elif guard >= 20000:
            self._fail("bot loop guard tripped")

    def _fail(self, msg):
        self.stuck = True
        self.error = msg
        self.say("sys", "Error: " + msg, "end")

    def _finish(self):
        w = self.env.winner
        if w == 0:
            self.say("you", "Victory", "end")
        elif w == 1:
            self.say("opp", "Defeat", "end")
        else:
            self.say("sys", "Draw", "end")

    def answer(self, indices):
        """Human answer. Raises ValueError with the engine's message when illegal."""
        if self.sel is None or self.stuck:
            raise ValueError("no decision pending")
        sel = self.sel
        if sel["player"] != ME:
            raise ValueError("not your decision")
        n = len(sel["option"])
        indices = [int(i) for i in indices]
        if any(i < 0 or i >= n for i in indices):
            raise ValueError("option index out of range")
        if not self.allows_repeats(sel) and len(set(indices)) != len(indices):
            raise ValueError("duplicate picks")
        if not (sel["minCount"] <= len(indices) <= sel["maxCount"]) and not (sel["maxCount"] <= 1 and n and len(indices) == 1):
            raise ValueError("pick between %d and %d" % (sel["minCount"], sel["maxCount"]))
        try:
            self._apply(indices, ME)
        except BaseException as e:  # noqa: BLE001
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            raise ValueError(str(e) or "illegal answer") from None
        self._run_bot()

    # ---- log -------------------------------------------------------------
    # Log lines are terse, simulator style: {"who": you|opp|sys, "kind": ..., "text": ...}.
    # kind: turn (turn header), act (an action), dmg, heal, ko, prize, cond, evo, info, end.
    def say(self, who, text, kind="act"):
        self.log.append({"n": len(self.log), "turn": self.state["turn"] if getattr(self, "state", None) else 0,
                         "who": who, "kind": kind, "text": text})

    def name_of(self, cid):
        return self.cards.card(ref_of(cid))["name"]

    def _slot(self, p, area, idx):
        pl = self.state["players"][p]
        if area == 4:
            return pl["active"]
        if idx is not None and idx < len(pl["bench"]):
            return pl["bench"][idx]
        return None

    def _slot_name(self, p, area, idx):
        top = self._top(self._slot(p, area, idx))
        if top:
            return self.name_of(top)
        return "Active" if area == 4 else "Bench %d" % ((idx or 0) + 1)

    def _top(self, slot):
        if not slot:
            return None
        pk = [c for c in slot["cards"] if c not in slot.get("energies", []) and c not in slot.get("tools", [])]
        # The stack is stored base -> top; only Pokémon cards remain after dropping attachments.
        return pk[-1] if pk else None

    def _ability_name(self, p, area, idx):
        top = self._top(self._slot(p, area, idx))
        if not top:
            return "Ability"
        powers = self.cards.card(ref_of(top)).get("powers") or []
        return powers[0]["name"] if powers else "Ability"

    def _attack_of(self, o, who):
        card = self.serials.get(o["serial"]) if o["serial"] is not None else None
        if not card:
            card = self._top(self.state["players"][who]["active"])
        c = self.cards.card(ref_of(card)) if card else None
        atk = None
        if c and c.get("attacks") and o["attackId"] is not None and o["attackId"] < len(c["attacks"]):
            atk = c["attacks"][o["attackId"]]
        return c, atk

    def _describe_option(self, sel, o, who):
        k = o["type"]
        card = self.serials.get(o["serial"]) if o["serial"] is not None else None
        cn = self.name_of(card) if card else None
        po = o["playerIndex"] if o["playerIndex"] is not None else who
        if k == 0:
            return str(o["number"])
        if k == 1:
            return "Yes"
        if k == 2:
            return "No"
        if k == 7:
            if card and self.cards.card(ref_of(card)).get("super") == "pokemon":
                return "%s → Bench" % cn
            return cn or "Play"
        if k == 8:
            tgt = self._slot_name(po, o["inPlayArea"], o["inPlayIndex"]) if o["inPlayArea"] else "?"
            return "%s → %s" % (cn, tgt)
        if k == 9:
            tgt = self._slot_name(po, o["inPlayArea"], o["inPlayIndex"]) if o["inPlayArea"] else "?"
            return "%s ⇒ %s" % (tgt, cn)
        if k == 10:
            return "%s · %s" % (self._slot_name(po, o["area"], o["index"]), self._ability_name(po, o["area"], o["index"]))
        if k == 12:
            return "Retreat → %s" % self._slot_name(who, 5, o["index"])
        if k == 13:
            c, atk = self._attack_of(o, who)
            if atk:
                return "%s · %s" % (c["name"], atk["name"])
            return "Attack"
        if k == 14:
            return "End turn"
        if k == 15:
            return "Stadium" if o["area"] == 7 else "Trainer ability"
        if o["area"] in (4, 5) and o["inPlayArea"] is None and k in (3, 4, 5, 6) and card is None:
            return self._slot_name(po, o["area"], o["index"])
        parts = []
        if cn:
            parts.append(cn)
        elif o["area"] == 6:
            parts.append("Prize %d" % (o["index"] + 1))
        elif o["area"] in (4, 5):
            parts.append(self._slot_name(po, o["area"], o["index"]))
        if o["inPlayArea"]:
            parts.append("→ " + self._slot_name(po, o["inPlayArea"], o["inPlayIndex"]))
        return " ".join(parts) or KIND.get(k, "option")

    def _describe_answer(self, sel, indices, who):
        """Terse log text for an answer, or None for nothing worth logging."""
        self._just_played = set()
        self._last_kind = None
        if indices is None:
            return None
        try:
            opts = [sel["option"][i] for i in indices]
            if sel["type"] == 0:
                if not opts:
                    return None
                o = opts[0]
                self._last_kind = o["type"]
                if o["type"] == 14:
                    return None  # the next turn header says it
                if o["type"] == 7 and o["serial"] is not None:
                    self._just_played = {self.serials.get(o["serial"])}
                return self._describe_option(sel, o, who)
            if sel["context"] == 38 and self.state["phase"] <= 1 and opts:
                # Only a player whose opponent mulliganed is offered extra draws.
                self.say("opp" if who == ME else "you", "Mulligan", "info")
                return "Draw +%s" % opts[0]["number"]
            if sel["context"] in (13, 41):
                return None  # damage lines / the turn header say it
            if opts and all(o["area"] == 6 for o in opts):
                return None  # the prize line says it
            hidden = who == OPP and any(o["area"] in (1, 2, 6, 12, None) and o["serial"] is not None and o["type"] in (3, 4, 5, 6) for o in opts)
            ctx = SHORT_CONTEXT.get(sel["context"]) or CONTEXT.get(sel["context"], "Choice")
            if self.state["phase"] <= 1 and sel["context"] in (1, 2):
                return None  # setup placement is face down
            if hidden:
                return "%s: %d card%s" % (ctx, len(opts), "" if len(opts) == 1 else "s")
            if not opts:
                return None
            # Named here, so the diff need not announce these cards entering play.
            self._just_played = {self.serials.get(o["serial"]) for o in opts if o["serial"] is not None}
            if all(o["type"] in (1, 2) for o in opts):
                yn = self._describe_option(sel, opts[0], who)
                return "%s: %s" % (ctx, yn) if sel["context"] in (42, 43) else yn
            names = [self._describe_option(sel, o, who) for o in opts]
            if len(names) > 1 and len(set(names)) < len(names):
                cnt = {}
                for n in names:
                    cnt[n] = cnt.get(n, 0) + 1
                names = ["%s ×%d" % (n, c) if c > 1 else n for n, c in cnt.items()]
            if len(names) > 4:
                return "%s: %d" % (ctx, len(opts))
            return "%s: %s" % (ctx, ", ".join(names))
        except Exception:  # noqa: BLE001 (never let logging break a game)
            return None

    def _snapshot(self):
        st = self.state
        snap = {"turn": st["turn"], "active": st["activePlayer"], "slots": {}, "prizes": [], "zones": [],
                "phase": st["phase"]}
        for pi, p in enumerate(st["players"]):
            snap["prizes"].append(sum(1 for pr in p["prizes"] if pr))
            snap["zones"].append(set(p["discard"]) | set(p["lostzone"]))
            for s in [p["active"]] + p["bench"]:
                if s["cards"]:
                    base = s["cards"][0]
                    card = self.cards.card(ref_of(self._top(s) or base))
                    hp = (s.get("hp") or card.get("hp", 0)) + s.get("hpBonus", 0)
                    snap["slots"][base] = {"p": pi, "name": card["name"], "hp": hp,
                                           "damage": s.get("damage", 0), "cond": set(s.get("specialConditions", [])),
                                           "cards": list(s["cards"])}
        return snap

    def _diff_log(self):
        old, new = self.prev, self._snapshot()
        self.prev = new
        if self.state["phase"] <= 1:
            return  # setup: Pokémon are placed face down
        for base, o in old["slots"].items():
            who = "you" if o["p"] == ME else "opp"
            n = new["slots"].get(base)
            if n is None:
                gone = [c for c in o["cards"] if any(c in zs for zs in new["zones"])]
                if gone:
                    self.say(who, "%s Knocked Out" % o["name"], "ko")
                else:
                    self.say(who, "%s left play" % o["name"], "info")
                continue
            if n["name"] != o["name"] and self._last_kind != 9:
                self.say(who, "%s ⇒ %s" % (o["name"], n["name"]), "evo")
            d = n["damage"] - o["damage"]
            if d > 0:
                self.say(who, "%s −%d · %d HP left" % (n["name"], d, max(0, n["hp"] - n["damage"])), "dmg")
            elif d < 0:
                self.say(who, "%s +%d HP" % (n["name"], -d), "heal")
            for c in sorted(n["cond"] - o["cond"]):
                self.say(who, "%s: %s" % (n["name"], CONDITION.get(c, "?")), "cond")
        for base, n in new["slots"].items():
            if base not in old["slots"] and base not in self._just_played and old["phase"] > 1:
                self.say("you" if n["p"] == ME else "opp", "%s → play" % n["name"], "info")
        for p in (ME, OPP):
            if new["prizes"][p] < old["prizes"][p]:
                k = old["prizes"][p] - new["prizes"][p]
                self.say("you" if p == ME else "opp", "Prize ×%d · %d left" % (k, new["prizes"][p]), "prize")
        # The header goes last: damage and Knock Outs above belong to the turn that just ended.
        if new["turn"] != old["turn"] or old["phase"] <= 1:
            self.say("you" if new["active"] == ME else "opp", "Turn %d" % new["turn"], "turn")
        self._just_played = set()
        self._last_kind = None

    # ---- views -----------------------------------------------------------
    def view(self, log_from=0):
        refs = set()

        def cid(c):
            refs.add(ref_of(c))
            return c

        st = self.state
        turn = st["turn"]
        players = []
        for pi, p in enumerate(st["players"]):
            prizes = []
            face_up = p.get("faceUpPrizes", [])
            for i, pr in enumerate(p["prizes"]):
                if not pr:
                    prizes.append({"state": "taken"})
                elif i < len(face_up) and face_up[i]:
                    prizes.append({"state": "up", "id": cid(pr[0])})
                else:
                    prizes.append({"state": "down"})
            # Setup places Pokémon face down: hide the opponent's until the game starts.
            hide = pi == OPP and st["phase"] <= 1
            pv = {
                "index": pi,
                "handCount": len(p["hand"]),
                "deckCount": len(p["deck"]),
                "discard": [cid(c) for c in p["discard"]],
                "lostzone": [cid(c) for c in p["lostzone"]],
                "prizes": prizes,
                "prizesLeft": sum(1 for x in prizes if x["state"] != "taken"),
                "active": self._slot_view(p["active"], cid, hide),
                "bench": [self._slot_view(s, cid, hide) for s in p["bench"]],
                "stadium": [cid(c) for c in p["stadium"]],
                "supporter": [cid(c) for c in p["supporter"]],
                "energyAttached": p.get("energyPlayedTurn") == turn,
                "supporterPlayed": p.get("supporterTurn") == turn,
                "retreated": p.get("retreatedTurn") == turn,
            }
            if pi == ME:
                # Real hand order is not exposed by canonical(); option indices are
                # matched by card serial, so any stable order works.
                pv["hand"] = [cid(c) for c in p["hand"]]
            players.append(pv)
        stadium = None
        for pv in players:
            if pv["stadium"]:
                stadium = {"id": pv["stadium"][0], "owner": pv["index"]}
        choice = self._choice_view(cid) if (self.sel is not None and self.sel["player"] == ME and not self.stuck) else None
        winner = self.env.winner if self.sel is None else None
        return {
            "over": self.sel is None or self.stuck,
            "winner": winner,
            "error": self.error,
            "turn": turn,
            "activePlayer": st["activePlayer"],
            "phase": st["phase"],
            "seed": self.seed,
            "decks": list(self.deck_ids),
            "deckNames": [_pretty(d) for d in self.deck_ids],
            "you": players[ME],
            "opp": players[OPP],
            "stadium": stadium,
            "choice": choice,
            "cards": {r: self.cards.card(r) for r in refs},
            "log": self.log[log_from:],
            "logLen": len(self.log),
        }

    def _slot_view(self, s, cid, hide=False):
        if not s["cards"]:
            return None
        if hide:
            return {"hidden": True}
        energies, tools = s.get("energies", []), s.get("tools", [])
        stack = [c for c in s["cards"] if c not in energies and c not in tools]
        top = stack[-1] if stack else s["cards"][0]
        card = self.cards.card(ref_of(top))
        hp = (s.get("hp") or card.get("hp", 0)) + s.get("hpBonus", 0)
        markers = sorted({m["name"] for m in s.get("markers", [])})
        return {
            "id": cid(top),
            "stack": [cid(c) for c in stack],
            "energies": [cid(c) for c in energies],
            "tools": [cid(c) for c in tools],
            "damage": s.get("damage", 0),
            "hp": hp,
            "conditions": [CONDITION.get(c, str(c)) for c in s.get("specialConditions", [])],
            "markers": markers,
        }

    def _choice_view(self, cid):
        sel = self.sel
        opts = []
        for i, o in enumerate(sel["option"]):
            card = self.serials.get(o["serial"]) if o["serial"] is not None else None
            ov = {
                "i": i, "kind": o["type"], "kindName": KIND.get(o["type"], "?"),
                "area": o["area"], "areaName": AREA.get(o["area"]), "index": o["index"],
                "player": o["playerIndex"], "inPlayArea": o["inPlayArea"], "inPlayIndex": o["inPlayIndex"],
                "attackId": o["attackId"], "number": o["number"],
                "card": cid(card) if card else None,
                "label": self._describe_option(sel, o, ME),
            }
            opts.append(ov)
        return {
            "type": sel["type"], "typeName": SELECT_TYPE.get(sel["type"], "?"),
            "context": sel["context"], "contextName": CONTEXT.get(sel["context"], "Context %d" % sel["context"]),
            "title": SHORT_CONTEXT.get(sel["context"]) or CONTEXT.get(sel["context"], "Choose"),
            "min": sel["minCount"], "max": sel["maxCount"],
            "repeats": self.allows_repeats(sel),
            "options": opts,
        }
