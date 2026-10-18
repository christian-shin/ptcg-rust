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
# Named hidden cards take named identities in place; a card that isn't hidden is refused.
ids = env.unseen_cards(0, 1)
assert len(ids) == n
kept_n = len(kept)
env.set_unseen_assigned(0, 1, [(ids[0], a[0])] if kept_n == 0 else [])
try:
    env.set_unseen_assigned(0, 0, [(ids[0], a[0])])
    raise AssertionError("a card the owner can't hide was accepted")
except ValueError:
    pass
steps = 0
what_ifs = 0
while not env.done:
    # The what-if attack query at every turn decision (an error at any other decision, or for an attack the text forbids).
    try:
        r = env.attack_what_if(0, 0, 0)
        assert abs(sum(o["probability"] for o in r["outcomes"]) + r["unexplored"] - 1) < 1e-9, r
        what_ifs += 1
    except ValueError:
        pass
    env.step_random(steps)
    steps += 1
    assert steps < 100000, "game did not end"
assert what_ifs > 0, "no what-if query answered"
print("ok", sys.version.split()[0], "steps", steps, "winner", env.winner, "what-ifs", what_ifs)
