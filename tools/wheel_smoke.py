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
steps = 0
while not env.done:
    env.step_random(steps)
    steps += 1
    assert steps < 100000, "game did not end"
print("ok", sys.version.split()[0], "steps", steps, "winner", env.winner)
