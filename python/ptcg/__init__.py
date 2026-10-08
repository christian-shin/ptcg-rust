"""Pokémon TCG rules engine: Python interface.

    env = ptcg.Env(deck_a, deck_b, seed=0)
    sel = env.select()            # cabt-style dict: type, context, minCount, maxCount, option
    env.step([0])                 # answer with option indices
    obs = env.observe(0)          # float32 [OBS_SIZE] from player 0's view
    copy = env.clone()            # cheap full copy (mid-prompt included)
    copy.determinize(0, seed=7)   # re-deal hidden cards for a search rollout

    venv = ptcg.VecEnv(64, deck_a, deck_b, seed=0)
    obs, opts, counts = venv.observe(k_max=32)
    rewards, dones = venv.step(actions)
"""
import numpy as np

from ._ptcg import Env as _Env, VecEnv as _VecEnv, OBS_SIZE, OPTION_FEATURES

__all__ = ["Env", "VecEnv", "OBS_SIZE", "OPTION_FEATURES", "read_deck"]


def read_deck(path):
    """Read a decklist file ("4 Card Name SET" per line) into a list of names."""
    cards = []
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        n, name = line.split(" ", 1)
        cards += [name] * int(n)
    return cards


class Env:
    """One game. Wraps the Rust engine; attribute access falls through to it."""

    def __init__(self, deck_a=None, deck_b=None, seed=0, manual_chance=False, _inner=None):
        self._e = _inner if _inner is not None else _Env(deck_a, deck_b, seed, manual_chance)

    def __getattr__(self, name):
        return getattr(self._e, name)

    def observe(self, player):
        return np.frombuffer(self._e.observe_bytes(player), dtype=np.float32)

    def options(self):
        return np.frombuffer(self._e.option_bytes(), dtype=np.int32).reshape(-1, OPTION_FEATURES)

    def clone(self):
        return Env(_inner=self._e.clone())


class VecEnv:
    """N games against a random opponent; the learner is seat 0."""

    def __init__(self, n, deck_a, deck_b, seed=0):
        self._v = _VecEnv(n, deck_a, deck_b, seed)
        self.num_envs = n

    def observe(self, k_max=32):
        o, a, c = self._v.observe(k_max)
        n = self.num_envs
        return (
            np.frombuffer(o, dtype=np.float32).reshape(n, OBS_SIZE),
            np.frombuffer(a, dtype=np.int32).reshape(n, k_max, OPTION_FEATURES),
            np.frombuffer(c, dtype=np.int32),
        )

    @property
    def invalid_answers(self):
        return self._v.invalid_answers

    @property
    def stuck_games(self):
        """Games ended early (reward 0, done) because a prompt had no valid answer."""
        return self._v.stuck_games

    @property
    def aborted_games(self):
        """Games ended early (reward 0, done) because the engine panicked."""
        return self._v.aborted_games

    def step(self, actions):
        r, d = self._v.step([int(a) for a in actions])
        return np.frombuffer(r, dtype=np.float32), np.frombuffer(d, dtype=np.uint8).astype(bool)
