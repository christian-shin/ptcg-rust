#!/usr/bin/env python3
"""Local web client for playing the Rust engine. Run: python client/server.py

Endpoints (JSON):
  GET  /api/decks                      available decks (decks/meta-tl)
  GET  /api/deck?id=ID                 decklist (international names)
  POST /api/new    {deckA, deckB, seed?, bot?}   start a game (you are player 0)
  GET  /api/state?log=N                filtered state for player 0 (+ choice, log lines from N)
  GET  /api/choice                     the current decision when it is yours (or null)
  POST /api/answer {indices:[...]}     answer it; returns the new state
  POST /api/validate {indices:[...]}   dry-run an answer (no state change)
"""
import argparse
import json
import os
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import game  # noqa: E402
from policies import POLICIES  # noqa: E402

STATIC = os.path.join(HERE, "static")
MIME = {".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
        ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".png": "image/png"}


class App:
    """One game at a time, guarded by a lock."""

    def __init__(self, default_bot="greedy"):
        self.lock = threading.Lock()
        self.cards = game.CardDB()
        self.session = None
        self.default_bot = default_bot

    def new(self, body):
        s = game.GameSession(body["deckA"], body["deckB"], body.get("seed"),
                             body.get("bot") or self.default_bot, self.cards)
        self.session = s
        return s.view()

    def state(self, log_from=0):
        if self.session is None:
            return {"none": True}
        return self.session.view(log_from)


def make_handler(app):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt, *args):
            if os.environ.get("CLIENT_VERBOSE"):
                super().log_message(fmt, *args)

        def _send(self, code, payload, ctype="application/json; charset=utf-8"):
            data = payload if isinstance(payload, bytes) else json.dumps(payload).encode()
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(data)

        def _body(self):
            n = int(self.headers.get("Content-Length") or 0)
            return json.loads(self.rfile.read(n) or b"{}") if n else {}

        def do_GET(self):
            u = urlparse(self.path)
            q = parse_qs(u.query)
            try:
                if u.path == "/api/decks":
                    return self._send(200, {"decks": game.list_decks(), "bots": sorted(POLICIES), "defaultBot": app.default_bot})
                if u.path == "/api/deck":
                    return self._send(200, {"cards": game.deck_listing(q["id"][0], app.cards)})
                if u.path == "/api/state":
                    with app.lock:
                        return self._send(200, app.state(int(q.get("log", ["0"])[0])))
                if u.path == "/api/choice":
                    with app.lock:
                        v = app.state()
                        return self._send(200, {"choice": v.get("choice"), "cards": v.get("cards", {})})
                return self._static(u.path)
            except Exception as e:  # noqa: BLE001
                return self._send(400, {"error": str(e)})

        def do_POST(self):
            u = urlparse(self.path)
            try:
                body = self._body()
                with app.lock:
                    if u.path == "/api/new":
                        return self._send(200, app.new(body))
                    if app.session is None:
                        return self._send(400, {"error": "no game; POST /api/new first"})
                    if u.path == "/api/answer":
                        try:
                            app.session.answer(body.get("indices", []))
                        except ValueError as e:
                            return self._send(409, {"error": str(e)})
                        return self._send(200, app.session.view(int(body.get("log", 0))))
                    if u.path == "/api/validate":
                        return self._send(200, {"error": app.session.validate(body.get("indices", []))})
                return self._send(404, {"error": "not found"})
            except Exception as e:  # noqa: BLE001
                return self._send(400, {"error": str(e)})

        def _static(self, path):
            if path in ("", "/"):
                path = "/index.html"
            full = os.path.normpath(os.path.join(STATIC, path.lstrip("/")))
            if not full.startswith(STATIC + os.sep) or not os.path.isfile(full):
                return self._send(404, {"error": "not found"})
            with open(full, "rb") as f:
                self._send(200, f.read(), MIME.get(os.path.splitext(full)[1], "application/octet-stream"))

    return Handler


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--bot", default="greedy", choices=sorted(POLICIES))
    a = ap.parse_args()
    app = App(a.bot)
    srv = ThreadingHTTPServer((a.host, a.port), make_handler(app))
    print("PTCG client on http://%s:%d" % (a.host if a.host != "0.0.0.0" else "localhost", a.port))
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
