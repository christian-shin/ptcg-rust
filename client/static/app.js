"use strict";
/* PTCG local client. Plain JS, no build step. All card data comes from the server;
   this file only renders the player-0 view and turns clicks into engine answers. */

const $ = (s) => document.querySelector(s);
function h(tag, attrs, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v === undefined || v === null || v === false) continue;
    if (k === "class") el.className = v;
    else if (k.startsWith("on")) el.addEventListener(k.slice(2), v);
    else if (k === "text") el.textContent = v;
    else el.setAttribute(k, v === true ? "" : v);
  }
  for (const kid of kids.flat()) {
    if (kid === null || kid === undefined || kid === false) continue;
    el.append(kid.nodeType ? kid : document.createTextNode(String(kid)));
  }
  return el;
}

const ME = 0, OPP = 1;
const cards = {};            // ref -> printed card data (server-built, international names)
let S = null;                // latest filtered view
let logLines = [];
let ui = fresh();
let showImages = false;
try { showImages = localStorage.getItem("ptcg.images") === "1"; } catch (e) { /* storage may be blocked */ }

function fresh() { return { pending: null, picks: [], counts: {}, minimized: false, err: "", menu: null, overDismissed: false }; }

/* ---------------- API ---------------- */
async function api(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
  const j = await r.json();
  return { ok: r.ok, status: r.status, body: j };
}

function applyView(v, replaceLog) {
  Object.assign(cards, v.cards || {});
  if (replaceLog) logLines = [];
  for (const l of v.log || []) logLines.push(l);
  S = v;
}

async function answer(indices) {
  ui.err = "";
  const r = await api("/api/answer", { indices, log: logLines.length });
  if (!r.ok) {
    ui.err = r.body.error || "rejected";
    if (!S.choice || modeOf().kind === "main") toast(ui.err);
    render();
    return;
  }
  ui = Object.assign(fresh(), { overDismissed: ui.overDismissed });
  applyView(r.body, false);
  render();
}

async function boot() {
  const r = await api("/api/state?log=0");
  if (r.ok && !r.body.none) { applyView(r.body, true); render(); } else { render(); openNewGame(true); }
}

/* ---------------- helpers ---------------- */
const cardOf = (id) => cards[id.split("#")[0]] || { name: id, super: "unknown" };
const keyOf = (p, area, idx) => (area === 4 ? p + ":a" : p + ":b" + idx);
function toast(msg) {
  const t = $("#toast"); t.textContent = msg; t.hidden = false;
  clearTimeout(toast.t); toast.t = setTimeout(() => (t.hidden = true), 3500);
}
function chips(letters) {
  return (letters || []).map((l) => h("span", { class: "energy t-" + l, text: l }));
}
function dmgText(a) { return a.damage ? String(a.damage) + (a.calc || "") : ""; }

/* ---------------- card rendering ---------------- */
function faceEl(id, big) {
  const c = cardOf(id);
  const main = (c.types && c.types[0]) || "";
  const cls = "card " + (c.super === "pokemon" ? "pokemon main-" + main : c.super === "energy" ? "energy-c" : c.super === "trainer" ? "trainer" : "") + (big ? " big" : "");
  const el = h("div", { class: cls });
  const head = h("div", { class: "c-head" }, h("span", { class: "c-name", text: c.name }));
  if (c.super === "pokemon") head.append(h("span", { class: "c-hp" }, "HP " + c.hp + " ", chips(c.types)));
  el.append(head);
  if (c.super === "pokemon") {
    el.append(h("div", { class: "c-sub", text: c.stage + (c.evolvesFrom ? " · from " + c.evolvesFrom : "") + (c.ex ? " · ex" : "") }));
    const body = h("div", { class: "c-body" });
    for (const p of c.powers || []) {
      body.append(h("div", { class: "abil-line", text: "Ability: " + p.name }));
      if (big && p.text) body.append(h("div", { class: "txt", text: p.text }));
    }
    for (const a of c.attacks || []) {
      body.append(h("div", { class: "atk-line" }, chips(a.cost), h("span", { class: "an", text: a.name }), h("span", { class: "ad", text: dmgText(a) })));
      if (big && a.text) body.append(h("div", { class: "txt", text: a.text }));
    }
    el.append(body);
    const foot = h("div", { class: "c-foot" });
    foot.append(h("span", {}, "Weak ", c.weakness.length ? c.weakness.join(" ") : "-"));
    foot.append(h("span", {}, "Res ", c.resistance.length ? c.resistance.join(" ") : "-"));
    foot.append(h("span", {}, "Retreat ", c.retreat.length ? chips(c.retreat) : "0"));
    el.append(foot);
  } else if (c.super === "trainer") {
    el.append(h("div", { class: "c-sub", text: "Trainer · " + c.trainerType }));
    el.append(h("div", { class: "c-body" }, h("div", { class: "txt", text: c.text || "" })));
  } else if (c.super === "energy") {
    el.append(h("div", { class: "c-sub" }, c.energyType + " Energy · provides ", chips(c.provides)));
    el.append(h("div", { class: "c-body" }, h("div", { class: "txt", text: c.energyType === "Special" ? c.text : "" })));
  }
  if (showImages && c.image) {
    const img = h("img", { src: c.image.replace("%s", big ? "LG" : "SM"), loading: "lazy", alt: c.name });
    img.addEventListener("error", () => { img.remove(); el.classList.remove("hasimg"); });
    el.classList.add("hasimg"); el.append(img);
  }
  el.addEventListener("mouseenter", () => showPreview(id));
  return el;
}
function backEl() { return h("div", { class: "card back" }); }
function showPreview(id) { $("#preview").replaceChildren(faceEl(id, true)); }

/* ---------------- mode ---------------- */
function modeOf() {
  const ch = S && S.choice;
  if (!ch) return { kind: "none" };
  if (ch.type === 0) return { kind: "main" };
  const o = ch.options;
  if (!o.length) return { kind: "empty" };
  if (o.every((x) => x.kind === 1 || x.kind === 2)) return { kind: "yesno" };
  if (ch.type === 8 && o.every((x) => x.kind === 0)) return { kind: "count" };
  if (ch.type === 6) return { kind: "attack" };
  const boardOK = o.every((x) => x.kind === 3 && (x.area === 4 || x.area === 5) && x.inPlayArea === null);
  if (boardOK) {
    const keys = o.map((x) => keyOf(x.player === null ? ME : x.player, x.area, x.index));
    if (ch.repeats || new Set(keys).size === keys.length) return { kind: "board", keys };
  }
  return { kind: "list" };
}

/* Map slotKey -> options currently targetable by a click on that slot. */
function targetMap() {
  const m = new Map();
  const add = (k, o) => { if (!m.has(k)) m.set(k, []); m.get(k).push(o); };
  const md = modeOf();
  if (md.kind === "main" && ui.pending) {
    for (const o of ui.pending.opts) {
      if (o.kind === 12) add(keyOf(ME, 5, o.index), o);
      else if (o.inPlayArea) add(keyOf(o.player === null ? ME : o.player, o.inPlayArea, o.inPlayIndex), o);
    }
  } else if (md.kind === "board") {
    S.choice.options.forEach((o, n) => add(md.keys[n], o));
  }
  return m;
}

/* ---------------- board ---------------- */
function slotEl(pi, area, idx, sv, targets) {
  if (!sv) return h("div", { class: "empty-slot", text: area === 4 ? "Active" : "Bench " + (idx + 1) });
  const key = keyOf(pi, area, idx);
  const tg = targets.get(key);
  const md = modeOf();
  const wrap = h("div", { class: "slot" + (tg ? " target" : "") });
  const cw = h("div", { class: "cardwrap" }, faceEl(sv.id));
  if (sv.damage) cw.append(h("div", { class: "dmg", text: "-" + sv.damage }));
  if (sv.stack.length > 1) cw.append(h("div", { class: "stack-n", text: "x" + sv.stack.length }));
  // Selection badge (board prompts).
  if (md.kind === "board" && tg) {
    const o = tg[0];
    const n = ui.counts[o.i] || (ui.picks.includes(o.i) ? ui.picks.indexOf(o.i) + 1 : 0);
    if (n) { wrap.classList.add("chosen"); cw.append(h("div", { class: "pick-badge", text: S.choice.repeats ? "+" + n : n })); }
  }
  if (tg) {
    cw.addEventListener("click", (e) => { e.stopPropagation(); slotClick(key, tg); });
    cw.addEventListener("contextmenu", (e) => { e.preventDefault(); if (md.kind === "board" && S.choice.repeats) { const o = tg[0]; if (ui.counts[o.i]) { ui.counts[o.i]--; render(); } } });
  }
  wrap.append(cw);
  const left = Math.max(0, sv.hp - sv.damage);
  const pct = sv.hp ? Math.round((100 * left) / sv.hp) : 0;
  wrap.append(h("div", { class: "hpbar" }, h("i", { class: pct <= 25 ? "low" : pct <= 55 ? "mid" : "", style: "width:" + pct + "%" })));
  wrap.append(h("div", { class: "hptxt", text: left + " / " + sv.hp + " HP" }));
  const att = h("div", { class: "attach" });
  for (const e of sv.energies) {
    const c = cardOf(e);
    const lab = (c.provides || ["C"]).join("");
    const p = h("span", { class: "energy t-" + (c.provides && c.provides[0] || "C"), title: c.name, text: lab.length > 2 ? lab.length : lab });
    if (c.energyType === "Special") p.style.outline = "1px solid #fff8";
    p.addEventListener("mouseenter", () => showPreview(e));
    att.append(p);
  }
  for (const t of sv.tools) { const p = h("span", { class: "pill tool", text: cardOf(t).name }); p.addEventListener("mouseenter", () => showPreview(t)); att.append(p); }
  for (const c of sv.conditions) att.append(h("span", { class: "pill cond", text: c }));
  for (const m of sv.markers) att.append(h("span", { class: "pill mark", text: m }));
  wrap.append(att);
  if (pi === ME && md.kind === "main" && !ui.pending) wrap.append(slotActions(area, idx, sv));
  return wrap;
}

function slotActions(area, idx, sv) {
  const acts = h("div", { class: "acts" });
  const ch = S.choice, c = cardOf(sv.id);
  let ab = 0;
  for (const o of ch.options) {
    if (o.kind === 13 && area === 4) {
      const a = (c.attacks || [])[o.attackId] || { name: "Attack", cost: [] };
      acts.append(h("button", { class: "atk", title: a.text || "", onclick: () => answer([o.i]) }, "Attack: " + a.name + (a.damage ? " " + dmgText(a) : "")));
    } else if (o.kind === 10 && o.area === area && (area === 4 || o.index === idx)) {
      const p = (c.powers || [])[ab++];
      acts.append(h("button", { class: "abil", title: p ? p.text : "", onclick: () => answer([o.i]) }, "Ability: " + (p ? p.name : "use")));
    }
  }
  if (area === 4) {
    const rets = ch.options.filter((o) => o.kind === 12);
    if (rets.length) acts.append(h("button", { class: "ret", onclick: () => { ui.pending = { label: "retreat to", opts: rets }; render(); } }, "Retreat..."));
  }
  return acts;
}

function slotClick(key, tg) {
  const md = modeOf();
  if (md.kind === "main" && ui.pending) {
    if (tg.length === 1) return answer([tg[0].i]);
    ui.menu = { title: "Choose action", opts: tg }; return render();
  }
  if (md.kind === "board") {
    const o = tg[0], ch = S.choice;
    if (ch.repeats) {
      const total = Object.values(ui.counts).reduce((a, b) => a + b, 0);
      if (total < ch.max) ui.counts[o.i] = (ui.counts[o.i] || 0) + 1;
    } else if (ui.picks.includes(o.i)) ui.picks = ui.picks.filter((x) => x !== o.i);
    else if (ch.max <= 1) ui.picks = [o.i];
    else if (ui.picks.length < ch.max) ui.picks.push(o.i);
    render();
  }
}

function pileEl(label, n, onclick) {
  return h("div", { class: "pile" + (onclick ? " click" : ""), onclick }, label + " ", h("b", { text: n }));
}
function prizesEl(pv) {
  return h("div", { class: "prizes" }, h("span", {}, "Prizes"), pv.prizes.map((p) => {
    const el = h("div", { class: "prize " + (p.state === "taken" ? "taken" : p.state === "up" ? "up" : "") });
    if (p.state === "up") el.addEventListener("mouseenter", () => showPreview(p.id));
    return el;
  }));
}
function viewPile(title, ids) {
  const v = $("#viewer");
  v.hidden = false;
  v.replaceChildren(h("div", { class: "modal" },
    h("header", {}, h("h2", { text: title + " (" + ids.length + ")" }), h("button", { onclick: () => (v.hidden = true) }, "Close")),
    h("div", { class: "body" }, h("div", { class: "tiles" }, ids.length ? ids.map((id) => h("div", { class: "tile" }, faceEl(id), h("div", { class: "cap", text: cardOf(id).name }))) : h("span", { class: "dim", text: "Empty" })))));
  v.onclick = (e) => { if (e.target === v) v.hidden = true; };
}
function infobar(pv, who) {
  const bar = h("div", { class: "infobar" }, h("span", { class: "name", text: who }));
  bar.append(pileEl("Hand", pv.handCount));
  bar.append(pileEl("Deck", pv.deckCount));
  bar.append(pileEl("Discard", pv.discard.length, () => viewPile(who + " discard", pv.discard)));
  if (pv.lostzone.length) bar.append(pileEl("Lost zone", pv.lostzone.length, () => viewPile(who + " lost zone", pv.lostzone)));
  bar.append(prizesEl(pv));
  if (pv.index === OPP) { const b = h("div", { class: "prizes" }); for (let i = 0; i < Math.min(pv.handCount, 12); i++) { const c = backEl(); c.style.cssText = "width:16px;height:22px;border-width:1px"; c.classList.add("mini"); b.append(c); } bar.append(b); }
  return bar;
}

function renderBoard() {
  const targets = targetMap();
  const board = $("#board");
  board.replaceChildren();
  const mkSide = (pv, who, cls, activeFirst) => {
    const act = h("div", { class: "row center" }, slotEl(pv.index, 4, 0, pv.active, targets));
    const bench = h("div", { class: "bench" }, pv.bench.map((s, i) => slotEl(pv.index, 5, i, s, targets)));
    const info = infobar(pv, who);
    return h("div", { class: "side-area " + cls }, activeFirst ? [act, bench, info] : [info, bench, act]);
  };
  board.append(mkSide(S.opp, "Opponent", "opp", false));
  const mid = h("div", { class: "mid" });
  const stadium = h("div", { class: "stadium-box" }, h("div", { class: "zone-label", text: "Stadium" }));
  if (S.stadium) stadium.append(h("div", { class: "tile" }, faceEl(S.stadium.id), h("div", { class: "cap", text: S.stadium.owner === ME ? "yours" : "opponent's" })));
  else stadium.append(h("div", { class: "dim", text: "none" }));
  mid.append(stadium);
  for (const [pv, who] of [[S.opp, "Opponent"], [S.you, "You"]]) {
    if (pv.supporter.length) mid.append(h("div", { class: "stadium-box" }, h("div", { class: "zone-label", text: who + " Supporter" }), faceEl(pv.supporter[0])));
  }
  board.append(mid);
  board.append(mkSide(S.you, "You", "you", true));
  board.onclick = () => { if (ui.pending) { ui.pending = null; render(); } };
}

/* ---------------- hand ---------------- */
function renderHand() {
  const hand = $("#hand");
  hand.replaceChildren();
  const md = modeOf();
  const ch = S.choice;
  const sorted = [...S.you.hand];
  sorted.sort((a, b) => { const A = cardOf(a), B = cardOf(b); return (A.super + A.name).localeCompare(B.super + B.name); });
  for (const id of sorted) {
    const el = faceEl(id);
    if (md.kind === "main") {
      const opts = ch.options.filter((o) => o.area === 2 && o.card === id);
      if (opts.length) {
        el.classList.add("playable");
        if (ui.pending && ui.pending.cardId === id) el.classList.add("pending");
        el.addEventListener("click", (e) => { e.stopPropagation(); handClick(id, opts); });
      }
    }
    hand.append(el);
  }
  if (!sorted.length) hand.append(h("span", { class: "dim", text: "Your hand is empty." }));
}
function handClick(id, opts) {
  if (ui.pending && ui.pending.cardId === id) { ui.pending = null; return render(); }
  if (opts.length === 1) return answer([opts[0].i]);
  if (opts.every((o) => o.inPlayArea)) { ui.pending = { cardId: id, label: "target for " + cardOf(id).name, opts }; return render(); }
  ui.menu = { title: cardOf(id).name, opts }; render();
}

/* ---------------- action bar ---------------- */
function ctxName() { return S.choice.contextName; }
function renderActionBar() {
  const bar = $("#actionbar");
  bar.replaceChildren();
  const md = modeOf();
  const msg = h("div", { class: "msg" });
  bar.append(msg);
  if (S.over) { msg.append("Game over."); bar.append(h("button", { class: "primary", onclick: () => openNewGame(false) }, "New game")); return; }
  if (!S.choice) { msg.append("Waiting..."); return; }
  if (md.kind === "main") {
    if (ui.pending) {
      msg.append(h("b", { text: "Choose " + ui.pending.label }), " - click a highlighted Pokémon, or ");
      bar.append(h("button", { onclick: () => { ui.pending = null; render(); } }, "Cancel"));
      return;
    }
    msg.append("Your move: click a glowing hand card, or use the buttons under your Pokémon.");
    const ch = S.choice;
    for (const o of ch.options) {
      if (o.kind === 15) bar.append(h("button", { onclick: () => answer([o.i]) }, o.label));
      if (o.kind === 16) bar.append(h("button", { onclick: () => answer([o.i]) }, o.label));
    }
    const end = ch.options.find((o) => o.kind === 14);
    const canAttack = ch.options.some((o) => o.kind === 13);
    if (end) bar.append(h("button", { class: "primary", onclick: () => { if (!canAttack || confirm("You can still attack. End your turn without attacking?")) answer([end.i]); } }, "End turn"));
    const det = h("details", { class: "allacts" }, h("summary", { text: "All legal actions (" + ch.options.length + ")" }),
      h("div", { class: "listrow" }, ch.options.map((o) => h("button", { onclick: () => answer([o.i]) }, o.label))));
    bar.append(det);
  } else if (md.kind === "board") {
    const ch = S.choice;
    const n = ch.repeats ? Object.values(ui.counts).reduce((a, b) => a + b, 0) : ui.picks.length;
    msg.append(h("b", { text: ctxName() }), " - click highlighted Pokémon (" + rangeText(ch) + (ch.repeats ? "; click adds a counter, right-click removes" : "") + "). Selected: " + n);
    bar.append(h("button", { onclick: () => { ui.picks = []; ui.counts = {}; render(); } }, "Clear"));
    bar.append(h("button", { class: "primary", disabled: !validCount(ch, n), onclick: confirmBoard }, "Confirm"));
    if (ui.err) msg.append(h("div", { class: "dim", style: "color:var(--bad)", text: ui.err }));
  } else if (ui.minimized) {
    msg.append(h("b", { text: ctxName() }), " - decision pending");
    bar.append(h("button", { class: "primary", onclick: () => { ui.minimized = false; render(); } }, "Show choice"));
  } else {
    msg.append("Decision pending: " + ctxName());
  }
}
function rangeText(ch) { return ch.min === ch.max ? "exactly " + ch.max : ch.min + " to " + ch.max; }
function validCount(ch, n) { return n >= ch.min && n <= ch.max; }
function chosenIndices() {
  const ch = S.choice;
  if (ch.repeats) { const out = []; for (const [i, c] of Object.entries(ui.counts)) for (let k = 0; k < c; k++) out.push(Number(i)); return out; }
  return ui.picks;
}
function confirmBoard() { answer(chosenIndices()); }

/* ---------------- modals ---------------- */
function renderOverlay() {
  const ov = $("#overlay");
  const md = modeOf();
  ov.onclick = null;
  if (S.over && !ui.overDismissed) {
    const w = S.winner;
    const t = w === 0 ? "You win!" : w === 1 ? "The opponent wins" : "Game over";
    ov.hidden = false;
    ov.replaceChildren(h("div", { class: "modal narrow" }, h("div", { class: "body banner" }, h("h1", { text: t }), S.error ? h("div", { class: "dim", text: S.error }) : null,
      h("div", { class: "btnrow", style: "justify-content:center" }, h("button", { class: "primary", onclick: () => openNewGame(false) }, "New game"), h("button", { onclick: () => { ui.overDismissed = true; render(); } }, "Look at the board")))));
    return;
  }
  if (ui.menu) {
    ov.hidden = false;
    ov.replaceChildren(h("div", { class: "modal narrow" }, h("header", {}, h("h2", { text: ui.menu.title })),
      h("div", { class: "body" }, h("div", { class: "listrow" }, ui.menu.opts.map((o) => h("button", { onclick: () => answer([o.i]) }, o.label)))),
      h("footer", {}, h("button", { onclick: () => { ui.menu = null; render(); } }, "Cancel"))));
    return;
  }
  if (!S.choice || md.kind === "main" || md.kind === "board" || ui.minimized || S.over) { ov.hidden = true; ov.replaceChildren(); return; }
  const ch = S.choice;
  ov.hidden = false;
  const title = ch.contextName + (ch.typeName === "card" || ch.typeName === "energy" || ch.typeName === "attachedCard" ? "" : "");
  const head = h("header", {}, h("h2", { text: title }), h("span", { class: "dim", text: md.kind === "list" ? rangeText(ch) : "" }));
  const body = h("div", { class: "body" });
  const foot = h("footer", {});
  const minBtn = h("button", { onclick: () => { ui.minimized = true; render(); } }, "Show board");
  const modal = h("div", { class: "modal" }, head, body, foot);
  ov.replaceChildren(modal);
  if (md.kind === "empty") {
    body.append(h("div", { class: "dim", text: "There are no valid choices for this effect." }));
    foot.append(h("div", { class: "err", text: ui.err }), h("button", { class: "primary", onclick: () => answer([]) }, "Continue"));
  } else if (md.kind === "yesno") {
    body.append(h("div", { class: "btnrow" }, ch.options.map((o) => h("button", { class: o.kind === 1 ? "primary" : "", onclick: () => answer([o.i]) }, o.kind === 1 ? "Yes" : "No"))));
    foot.append(minBtn);
  } else if (md.kind === "count") {
    body.append(h("div", { class: "btnrow" }, ch.options.map((o) => h("button", { onclick: () => answer([o.i]) }, ch.context === 38 ? "Draw " + o.number : "#" + o.number))));
    foot.append(minBtn);
  } else if (md.kind === "attack") {
    body.append(h("div", { class: "listrow" }, ch.options.map((o) => {
      const c = o.card ? cardOf(o.card) : null;
      const a = c && c.attacks && c.attacks[o.attackId];
      return h("button", { onclick: () => answer([o.i]) }, a ? [c.name + ": ", chips(a.cost), " " + a.name + " " + dmgText(a)] : o.label);
    })));
    foot.append(minBtn);
  } else {
    renderListModal(ch, body, foot, minBtn);
  }
}

function renderListModal(ch, body, foot, minBtn) {
  const tiles = h("div", { class: "tiles" });
  const cnt = h("div", { class: "cnt" });
  const confirmBtn = h("button", { class: "primary", onclick: () => answer(chosenIndices()) }, "Confirm");
  const refresh = () => {
    const n = ch.repeats ? Object.values(ui.counts).reduce((a, b) => a + b, 0) : ui.picks.length;
    cnt.textContent = "Selected " + n + " (" + rangeText(ch) + ")" + (ch.repeats ? "" : "");
    confirmBtn.disabled = !validCount(ch, n);
  };
  const toggle = (o) => {
    if (ch.repeats) return;
    if (ui.picks.includes(o.i)) ui.picks = ui.picks.filter((x) => x !== o.i);
    else if (ch.max <= 1) ui.picks = [o.i];
    else if (ui.picks.length < ch.max) ui.picks.push(o.i);
    else return;
    paint();
  };
  const paint = () => {
    tiles.replaceChildren();
    for (const o of ch.options) {
      const sel = ui.picks.includes(o.i);
      const t = h("div", { class: "tile" + (sel ? " sel" : "") + (o.card ? "" : " wide"), onclick: () => toggle(o) });
      if (o.card) t.append(faceEl(o.card));
      else t.append(h("div", { class: "box", text: o.area === 6 ? "Prize " + (o.index + 1) : o.label }));
      t.append(h("div", { class: "cap", text: o.card ? o.label : (o.area === 6 ? "face down" : "") }));
      if (sel && !ch.repeats) t.append(h("div", { class: "pick-badge", text: ui.picks.indexOf(o.i) + 1 }));
      if (ch.repeats) {
        const c = ui.counts[o.i] || 0;
        const total = () => Object.values(ui.counts).reduce((a, b) => a + b, 0);
        t.append(h("div", { class: "ctr" },
          h("button", { onclick: (e) => { e.stopPropagation(); if (ui.counts[o.i]) { ui.counts[o.i]--; paint(); } } }, "-"),
          h("b", { text: c }),
          h("button", { onclick: (e) => { e.stopPropagation(); if (total() < ch.max) { ui.counts[o.i] = (ui.counts[o.i] || 0) + 1; paint(); } } }, "+")));
      }
      tiles.append(t);
    }
    refresh();
  };
  body.append(tiles);
  if (ch.context === 1 || ch.context === 2) body.prepend(h("div", { class: "dim", style: "margin-bottom:8px", text: ch.context === 1 ? "Setup: pick your Active Pokémon first; any further picks go to your Bench." : "Setup: choose Basic Pokémon to put on your Bench." }));
  if (ch.max > 1 && !ch.repeats) body.prepend(h("div", { class: "dim", style: "margin-bottom:8px", text: "Click in the order you want; the badge shows pick order (matters for ordering effects)." }));
  paint();
  foot.append(cnt, h("div", { class: "err", text: ui.err }), minBtn,
    h("button", { onclick: () => { ui.picks = []; ui.counts = {}; paint(); } }, "Clear"), confirmBtn);
}

/* ---------------- new game ---------------- */
async function openNewGame(first) {
  const d = (await api("/api/decks")).body;
  const ov = $("#overlay");
  let last = {};
  try { last = JSON.parse(localStorage.getItem("ptcg.decks") || "{}"); } catch (e) { /* ignore */ }
  const mk = (sel) => h("select", {}, d.decks.map((x) => h("option", { value: x.id, selected: x.id === sel }, x.name + " (" + x.count + ")")));
  const a = mk(last.a || "dragapult-ex"), b = mk(last.b || "raging-bolt-ex");
  const seed = h("input", { type: "number", placeholder: "random", style: "width:110px" });
  const bot = h("select", {}, d.bots.map((x) => h("option", { value: x, text: x })));
  const la = h("ul"), lb = h("ul");
  const fill = async (sel, ul) => { const r = await api("/api/deck?id=" + sel.value); ul.replaceChildren(...(r.body.cards || []).map((c) => h("li", { text: c.count + " " + c.name }))); };
  a.onchange = () => fill(a, la); b.onchange = () => fill(b, lb);
  fill(a, la); fill(b, lb);
  const go = h("button", { class: "primary", onclick: async () => {
    try { localStorage.setItem("ptcg.decks", JSON.stringify({ a: a.value, b: b.value })); } catch (e) { /* ignore */ }
    const r = await api("/api/new", { deckA: a.value, deckB: b.value, seed: seed.value === "" ? undefined : Number(seed.value), bot: bot.value });
    if (!r.ok) return toast(r.body.error || "failed");
    ui = fresh(); applyView(r.body, true); render();
  } }, "Start game");
  ov.hidden = false;
  ov.replaceChildren(h("div", { class: "modal" }, h("header", {}, h("h2", { text: "New game" }), h("span", { class: "dim", text: "You are player 1 (bottom)" })),
    h("div", { class: "body" }, h("div", { class: "newgame" },
      h("div", {}, h("div", { class: "zone-label", text: "Your deck" }), a, la),
      h("div", {}, h("div", { class: "zone-label", text: "Opponent deck" }), b, lb))),
    h("footer", {}, h("span", { class: "dim" }, "Seed "), seed, h("span", { class: "dim" }, "Bot "), bot, h("div", { class: "cnt" }),
      first ? null : h("button", { onclick: () => render() }, "Cancel"), go)));
}

/* ---------------- top level ---------------- */
function renderTop() {
  const t = $("#turn-ind"), b = $("#badges");
  b.replaceChildren();
  if (!S) { t.textContent = ""; return; }
  const mine = S.activePlayer === ME;
  const live = S.phase >= 2;
  if (S.over) { t.textContent = "Game over"; t.className = ""; }
  else if (!live) { t.textContent = "Setup"; t.className = ""; }
  else { t.textContent = "Turn " + S.turn + " - " + (mine ? "your turn" : "opponent's turn"); t.className = mine ? "mine" : "theirs"; }
  const flag = (txt, on) => b.append(h("span", { class: "badge" + (on ? " on" : "") }, txt));
  flag("Energy attached", live && mine && S.you.energyAttached);
  flag("Supporter played", live && mine && S.you.supporterPlayed);
  flag("Retreated", live && mine && S.you.retreated);
  b.append(h("span", { class: "badge", text: S.decks[0] + " vs " + S.decks[1] + " - seed " + S.seed }));
}
function renderLog() {
  const el = $("#log");
  const stick = el.scrollTop + el.clientHeight >= el.scrollHeight - 30;
  el.replaceChildren(...logLines.map((l) => h("div", { class: l.who, text: l.text })));
  if (stick) el.scrollTop = el.scrollHeight;
}
function render() {
  if (!S) { return; }
  renderTop(); renderBoard(); renderHand(); renderActionBar(); renderLog(); renderOverlay();
}

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") { if (!$("#viewer").hidden) $("#viewer").hidden = true; else if (ui.menu) { ui.menu = null; render(); } else if (ui.pending) { ui.pending = null; render(); } }
});
$("#btn-new").addEventListener("click", () => openNewGame(!S));
const cb = $("#opt-images");
cb.checked = showImages;
cb.addEventListener("change", () => { showImages = cb.checked; try { localStorage.setItem("ptcg.images", showImages ? "1" : "0"); } catch (e) { /* ignore */ } if (S) render(); });
boot();
