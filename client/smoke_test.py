#!/usr/bin/env python3
"""Plays one full game through the HTTP API: the bot plays seat 1 as in the real
client, and this script plays seat 0 with random answers (validated against the
engine through /api/validate). Asserts the game finishes.

    python client/smoke_test.py [--seed N] [--url http://localhost:8765]

Without --url it starts its own server on a free port.
"""
import argparse
import json
import os
import random
import sys
import threading
import urllib.request
from http.server import ThreadingHTTPServer

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)


def call(base, path, body=None):
    req = urllib.request.Request(base + path, data=None if body is None else json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        return e.code, json.load(e)


def random_answer(base, ch, rng):
    opts = ch["options"]
    n = len(opts)
    lo, hi = ch["min"], min(ch["max"], n)
    for _ in range(60):
        if hi <= 1:
            if n == 0 or (lo == 0 and rng.randrange(n + 1) == n):
                picks = []
            else:
                picks = [rng.randrange(n)]
        else:
            k = rng.randint(lo, hi)
            picks = [rng.randrange(n) for _ in range(k)] if ch["repeats"] else rng.sample(range(n), k)
        _, v = call(base, "/api/validate", {"indices": picks})
        if v["error"] is None:
            return picks
    raise AssertionError("no legal answer found for %s/%s" % (ch["typeName"], ch["contextName"]))


def play(base, seed, deck_a, deck_b, max_steps=20000):
    rng = random.Random(seed)
    code, st = call(base, "/api/new", {"deckA": deck_a, "deckB": deck_b, "seed": seed})
    assert code == 200, st
    steps = 0
    while not st["over"]:
        ch = st["choice"]
        assert ch is not None, "game not over but no choice for the human: %r" % st.get("error")
        picks = random_answer(base, ch, rng)
        code, st = call(base, "/api/answer", {"indices": picks})
        assert code == 200, (code, st, picks, ch["contextName"])
        steps += 1
        assert steps < max_steps, "game did not finish in %d human decisions" % max_steps
    return st, steps


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--url")
    ap.add_argument("--deck-a", default="dragapult-ex")
    ap.add_argument("--deck-b", default="raging-bolt-ex")
    a = ap.parse_args()
    srv = None
    base = a.url
    if not base:
        import server
        srv = ThreadingHTTPServer(("127.0.0.1", 0), server.make_handler(server.App()))
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        base = "http://127.0.0.1:%d" % srv.server_address[1]
    code, d = call(base, "/api/decks")
    assert code == 200 and d["decks"], d
    st, steps = play(base, a.seed, a.deck_a, a.deck_b)
    assert st["over"] and st["error"] is None, st["error"]
    print("OK: game finished after %d human decisions, turn %d, winner=%r, %d log lines"
          % (steps, st["turn"], st["winner"], st["logLen"]))
    if srv:
        srv.shutdown()


if __name__ == "__main__":
    main()
