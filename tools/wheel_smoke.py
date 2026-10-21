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
# Seat 1's hand grows by 2 cards from their deck (the total kept); a different total is refused with the total stated.
hand, deck = env.hand_and_deck_count(1)
env.resize_unseen(0, 1, hand + 2, deck - 2, 3)
assert env.hand_and_deck_count(1) == (hand + 2, deck - 2)
assert env.unseen_count(0, 1) == n
try:
    env.resize_unseen(0, 1, hand + 3, deck - 2, 3)
    raise AssertionError("a different total was accepted")
except ValueError as e:
    assert str(hand + deck) in str(e), e
# A snapshot puts the game back: the same state after 20 random steps, in this Env and in a clone of another state.
snap = env.snapshot()
before = env.state_hash()
other = env.clone()
for i in range(20):
    env.step_random(1000 + i)
assert env.state_hash() != before
env.restore(snap)
assert env.state_hash() == before and env.select() == other.select()
other.step_random(5)
other.restore(snap)
assert other.state_hash() == before and 1000 < snap.size < 64000, snap.size
steps = 0
what_ifs = 0
while not env.done:
    # The what-if attack query at every turn decision (an error at any other decision, or for an attack the text forbids).
    try:
        r = env.attack_what_if(0, 0, 0)
        assert abs(sum(o["probability"] for o in r["outcomes"]) + r["unexplored"] - 1) < 1e-9, r
        what_ifs += 1
        # With an extra Energy attached (a card of the player's own; refused when none is left).
        r = env.attack_what_if(0, 0, 0, extra_energy=["Metal Energy MEE 8"])
        assert all(z in ("Hand", "Deck", "Discard") for _, z in r["extra_attached"]), r
    except ValueError:
        pass
    env.step_random(steps)
    steps += 1
    assert steps < 100000, "game did not end"
assert what_ifs > 0, "no what-if query answered"
print("ok", sys.version.split()[0], "steps", steps, "winner", env.winner, "what-ifs", what_ifs)
# Every table rule on: a game plays to the end; an unknown rule name is refused.
try:
    ptcg.Env(a, b, seed=1, rules={"no_such_rule": True})
    raise AssertionError("an unknown table rule was accepted")
except ValueError:
    pass
on = ptcg.Env(a, b, seed=2, rules={n: True for n in ptcg.TABLE_RULES})
steps = 0
while not on.done:
    on.step_random(steps)
    steps += 1
    assert steps < 100000, "game with every table rule did not end"
print("ok table rules", list(ptcg.TABLE_RULES), "steps", steps, "winner", on.winner)
