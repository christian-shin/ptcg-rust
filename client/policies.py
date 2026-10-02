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


POLICIES = {"random": RandomPolicy}


def make_policy(name, seed=None):
    return POLICIES[name](seed)
