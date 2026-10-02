"""Opponent policies for the web client.

A policy is any object with `choose(session) -> list[int] | None`: option
indices answering the engine's current decision for seat 1, or None to let the
engine pick a uniformly random legal answer (`Env.step_random`; the picks are
then not reported, so the log shows only the visible effects).

`session` is the `GameSession` (client/game.py): `session.env` is the live
`ptcg.Env`, `session.sel` the current select dict, `session.state` the full,
unfiltered canonical state JSON. Policies play seat 1 and may look at
everything; nothing here is shown to the human. Probe candidate answers with
`session.try_answer(indices)` (runs on a clone, returns True if legal).

To plug in a smarter bot, subclass `Policy`, add it to `POLICIES`, and select
it with `--bot NAME` or the `"bot"` field of the new-game request.
"""
import random


class Policy:
    name = "base"

    def __init__(self, seed=None):
        self.rng = random.Random(seed)

    def choose(self, session):
        raise NotImplementedError


class RandomPolicy(Policy):
    """Uniformly random legal answers."""

    name = "random"

    def choose(self, session):
        sel = session.sel
        n = len(sel["option"])
        lo, hi = sel["minCount"], min(sel["maxCount"], n)
        repeats = session.allows_repeats(sel)
        if hi <= 1:
            # Single pick (or none): any option is legal; allow "nothing" when min is 0.
            if n == 0:
                return []
            if lo == 0 and self.rng.randrange(n + 1) == n:
                return []
            return [self.rng.randrange(n)]
        # Multi-pick: sample candidate answers and keep the first the engine accepts.
        for _ in range(40):
            k = self.rng.randint(lo, hi) if hi > lo else lo
            picks = ([self.rng.randrange(n) for _ in range(k)] if repeats
                     else sorted(self.rng.sample(range(n), min(k, n))))
            if session.try_answer(picks):
                return picks
        return None  # engine-side random (builds legal multi-picks one at a time)


class GreedyPolicy(RandomPolicy):
    """Random, but plays like it means it: on its turn it takes a few random
    non-attack actions (cards, Energy, abilities), then attacks whenever it can,
    instead of ending the turn at random. Other prompts are answered randomly."""

    name = "greedy"
    MAX_ACTIONS = 12

    def __init__(self, seed=None):
        super().__init__(seed)
        self.turn = None
        self.actions = 0

    def choose(self, session):
        sel = session.sel
        if sel["type"] != 0:
            return super().choose(session)
        turn = session.state["turn"]
        if turn != self.turn:
            self.turn, self.actions = turn, 0
        self.actions += 1
        opts = sel["option"]
        attacks = [i for i, o in enumerate(opts) if o["type"] == 13]
        end = [i for i, o in enumerate(opts) if o["type"] == 14]
        other = [i for i, o in enumerate(opts) if o["type"] not in (12, 13, 14)]
        # Attachments to the Active Pokémon first, so it can actually attack.
        to_active = [i for i in other if opts[i]["type"] == 8 and opts[i]["inPlayArea"] == 4]
        if to_active and self.rng.random() < 0.8:
            return [self.rng.choice(to_active)]
        if other and self.actions <= self.MAX_ACTIONS and self.rng.random() < 0.85:
            return [self.rng.choice(other)]
        if attacks:
            return [self.rng.choice(attacks)]
        if end:
            return [end[0]]
        return [self.rng.randrange(len(opts))] if opts else []


POLICIES = {"random": RandomPolicy, "greedy": GreedyPolicy}


def make_policy(name, seed=None):
    return POLICIES[name](seed)
