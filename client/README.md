# Local web client

Play the Rust engine in a browser. You are player 0 (bottom); the opponent is a bot (uniformly random legal answers by default).

## Build the bindings

```
cd python
maturin develop --release        # into the repo .venv  (or: maturin build, then install the wheel)
cd ..
```

The server needs `import ptcg` to work (`ptcg._ptcg` is the compiled module). It also reads `data/pool.json`, `data/twinleaf-cards.json`, and `decks/meta-tl/*.txt`. No extra Python dependencies (stdlib `http.server`).

## Run

```
.venv/bin/python client/server.py [--port 8765] [--bot random]
```

Open http://localhost:8765, pick both decks, and start. The "Card images" toggle loads card art from Limitless's CDN
(`limitlesstcg.nyc3.cdn.digitaloceanspaces.com/tpci/SET/SET_NNN_R_EN_SM|LG.png`); cards fall back to text if an image is missing.

## Smoke test

```
.venv/bin/python client/smoke_test.py [--seed N] [--url http://localhost:8765]
```

Plays a full game through the HTTP API (random answers for seat 0, the bot for seat 1) and asserts it finishes.

## Layout of the code

- `server.py` - HTTP API (`/api/decks`, `/api/new`, `/api/state`, `/api/choice`, `/api/answer`, `/api/validate`) and static files.
- `game.py` - `GameSession`: wraps `ptcg.Env`, filters hidden information, resolves bot decisions, builds card data and the log.
- `policies.py` - opponent policies. Add a class with `choose(session) -> indices | None` to `POLICIES` to plug in a smarter bot (`--bot NAME`).
- `static/` - single-page frontend (`index.html`, `style.css`, `app.js`), no build step.
