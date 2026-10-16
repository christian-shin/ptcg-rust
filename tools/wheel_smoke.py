"""Smoke test of an installed ptcg wheel: import, play one random game to the end."""
import os
import sys

import ptcg

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
a = ptcg.read_deck(os.path.join(root, "decks", "t1-00.txt"))
b = ptcg.read_deck(os.path.join(root, "decks", "t1-01.txt"))
env = ptcg.Env(a, b, seed=1)
assert env.select() is not None
assert env.observe(0).shape == (ptcg.OBS_SIZE,)
# Seat 1's cards unseen by seat 0 take deck a's identities (the kept ones first); a short list is refused.
n = env.unseen_count(0, 1)
kept = env.unseen_kept(0, 1)
names = kept + (a * 2)[: n - len(kept)]
try:
    env.set_unseen(0, 1, names[1:], 7)
    raise AssertionError("a short list was accepted")
except ValueError:
    pass
env.set_unseen(0, 1, names, 7)
assert env.unseen_count(0, 1) == n
steps = 0
while not env.done:
    env.step_random(steps)
    steps += 1
    assert steps < 100000, "game did not end"
print("ok", sys.version.split()[0], "steps", steps, "winner", env.winner)
