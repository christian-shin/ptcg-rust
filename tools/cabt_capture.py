#!/usr/bin/env python3
"""Capture observations, option lists and actions from the Kaggle "cabt" environment.

Runs N games (each a best-of-`bo` match) of make("cabt") with random legal agents (or the
"first" agent: always the first minCount options) and writes one JSON line per decision to
--out (JSONL). Needs kaggle-environments 1.33.0 and a platform whose cg library loads
(Linux x86_64 or Apple Silicon); the interpreter can be a container.

Record kinds (field "k"):
  config  : env configuration, decks, agent names, python/platform info
  step    : {game, n, seat, obs (the full dict the agent got), action}
  (the decks of each game are in its game_start record)
  env_steps: {game, status, reward} per env step (list of [seat0, seat1])
  final_obs: {game, obs: [seat0 observation, seat1 observation]} of the last env step
  end     : {game, statuses, rewards, result (wins0, wins1, draws), steps}

Usage: cabt_capture.py --deck0 a.csv --deck1 b.csv --games 10 --out out.jsonl [--bo 3]
       [--agents random,random] [--seed 1]
A deck file has 60 card ids, one per line; --deck0/--deck1 may be comma lists (one is chosen per game).
"""
import argparse, json, platform, random, sys


def load_deck(path):
    return [int(l) for l in open(path) if l.strip()]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--deck0", required=True)
    ap.add_argument("--deck1", required=True)
    ap.add_argument("--games", type=int, default=1)
    ap.add_argument("--bo", type=int, default=3)
    ap.add_argument("--agents", default="random,random")
    ap.add_argument("--seed", type=int, default=0, help="seeds the agents' choices only; the engine has no seed")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    from kaggle_environments import make
    import kaggle_environments
    pools = [[load_deck(p) for p in a.deck0.split(",")], [load_deck(p) for p in a.deck1.split(",")]]
    rng = random.Random(a.seed)
    names = a.agents.split(",")
    out = open(a.out, "w")
    first = True
    for g in range(a.games):
        decks = [rng.choice(pools[0]), rng.choice(pools[1])]  # a comma list in --deckN is a pool
        env = make("cabt", configuration={"decks": decks, "bo": a.bo})
        if first:
            out.write(json.dumps({"k": "config", "deck_files": [a.deck0, a.deck1], "configuration": dict(env.configuration), "agents": names, "seed": a.seed, "bo": a.bo,
                                  "kaggle_environments": kaggle_environments.__version__,
                                  "python": sys.version.split()[0], "platform": platform.platform()}) + "\n")
            first = False
        out.write(json.dumps({"k": "game_start", "game": g, "decks": decks}) + "\n")
        n = [0]

        def make_agent(seat, name):
            def agent(obs):
                obs = json.loads(json.dumps(obs))
                if obs["select"] is None:
                    act = decks[seat]
                elif name == "first":
                    act = list(range(obs["select"]["maxCount"]))
                else:
                    act = rng.sample(range(len(obs["select"]["option"])), obs["select"]["maxCount"])
                    # note: maxCount-sized sample as in the shipped random agent
                seat_i = obs["current"]["yourIndex"] if obs["current"] else seat
                out.write(json.dumps({"k": "step", "game": g, "n": n[0], "seat": seat_i, "obs": obs,
                                      "action": act}) + "\n")
                n[0] += 1
                return act
            return agent

        env.run([make_agent(i, names[i]) for i in range(2)])
        last = env.steps[-1]
        # per-step status/reward as the wrapper records them (index i = env step i)
        out.write(json.dumps({"k": "env_steps", "game": g,
                              "status": [[x["status"] for x in st] for st in env.steps],
                              "reward": [[x["reward"] for x in st] for st in env.steps]}) + "\n")
        # the observations of the final env step (what the wrapper holds when the match ends)
        out.write(json.dumps({"k": "final_obs", "game": g,
                              "obs": json.loads(json.dumps([x["observation"] for x in last]))}) + "\n")
        out.write(json.dumps({"k": "end", "game": g, "statuses": [s["status"] for s in last],
                              "rewards": [s["reward"] for s in last],
                              "result": getattr(env, "result", None), "steps": n[0]}) + "\n")
        out.flush()
    out.close()


if __name__ == "__main__":
    main()
