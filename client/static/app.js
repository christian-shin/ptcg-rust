"use strict";
/* PTCG local client. Plain JS, no build step. The server sends a filtered player-0 view;
   this file renders the table and turns clicks (or drags) into engine answers. */

const $ = (s) => document.querySelector(s);
function h(tag, attrs, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v === undefined || v === null || v === false) continue;
    if (k === "class") el.className = v;
    else if (k === "style") el.style.cssText = v;
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
const cards = {};              // ref -> printed card data (server-built, international names)
const badImg = new Set();      // image URLs that failed to load (render as text cards)
let S = null;                  // latest filtered view
let logLines = [];
let ui = fresh();
let fx = noFx();
let busy = false;
let showImages = true;
try { showImages = localStorage.getItem("ptcg.images") !== "0"; } catch (e) { /* storage may be blocked */ }

function fresh() { return { pending: null, picks: [], counts: {}, minimized: false, err: "", overDismissed: false, endArm: false, pop: null, modal: null }; }
function noFx() { return { hit: {}, heal: {}, enter: new Set(), evo: new Set(), draw: new Set(), prize: new Set(), banner: null }; }

/* ---------------- API ---------------- */
async function api(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
  let j = {};
  try { j = await r.json(); } catch (e) { j = { error: "bad response" }; }
  return { ok: r.ok, status: r.status, body: j };
}

function applyView(v, replaceLog) {
  Object.assign(cards, v.cards || {});
  fx = replaceLog ? noFx() : computeFx(S, v);
  if (replaceLog) logLines = [];
  for (const l of v.log || []) logLines.push(l);
  S = v;
}

/* Name of the card / attack / ability whose effect the next prompts belong to. */
let promptSrc = null;
function srcOf(o) {
  if (!o) return null;
  if (o.kind === 7 && o.card) return cardOf(o.card).name;
  if (o.kind === 13 || o.kind === 10 || o.kind === 15) return o.label.split(" · ").pop();
  return null;
}
async function answer(indices) {
  if (busy) return;
  if (S && S.choice && S.choice.type === 0) promptSrc = srcOf(S.choice.options[indices[0]]);
  busy = true;
  ui.err = "";
  closePop();
  let r;
  try { r = await api("/api/answer", { indices, log: logLines.length }); } finally { busy = false; }
  if (!r.ok) {
    ui.err = shortErr(r.body.error || "Rejected");
    if (!S.choice || modeOf().kind === "main") toast(ui.err);
    render();
    return;
  }
  ui = Object.assign(fresh(), { overDismissed: ui.overDismissed });
  applyView(r.body, false);
  if (!S.choice || S.choice.type === 0) promptSrc = null;
  render();
}
function shortErr(e) { return String(e).replace(/^.*?Error:\s*/, "").slice(0, 140); }

async function boot() {
  const r = await api("/api/state?log=0");
  if (r.ok && !r.body.none) { applyView(r.body, true); render(); } else { openNewGame(true); }
}

/* ---------------- helpers ---------------- */
const cardOf = (id) => cards[String(id).split("#")[0]] || { name: String(id), super: "unknown" };
const keyOf = (p, area, idx) => (area === 4 ? p + ":a" : p + ":b" + idx);
const optPlayer = (o) => (o.player === null || o.player === undefined ? ME : o.player);
function toast(msg) {
  const t = $("#toast"); t.textContent = msg; t.hidden = false;
  clearTimeout(toast.t); toast.t = setTimeout(() => (t.hidden = true), 3000);
}
function eIcons(letters, cls) { return (letters || []).map((l) => h("span", { class: "e e-" + l + (cls ? " " + cls : ""), text: l === "C" ? "" : "" })); }
function dmgText(a) { return a && a.damage ? String(a.damage) + (a.calc || "") : ""; }
function imgUrl(c, big) { return c.image ? c.image.replace("%s", big ? "LG" : "SM") : null; }
function topSlot(pv, area, idx) { return area === 4 ? pv.active : pv.bench[idx]; }

/* ---------------- effects (diff of consecutive views) ---------------- */
function slotsOf(v, who) {
  const m = new Map();
  const pv = v[who];
  [pv.active, ...pv.bench].forEach((s) => { if (s && !s.hidden) m.set(s.stack[0], s); });
  return m;
}
function computeFx(a, b) {
  const f = noFx();
  if (!a || !b || a.seed !== b.seed) return f;
  for (const who of ["you", "opp"]) {
    const A = slotsOf(a, who), B = slotsOf(b, who);
    for (const [base, s] of B) {
      const o = A.get(base);
      if (!o) { f.enter.add(base); continue; }
      if (s.damage > o.damage) f.hit[base] = s.damage - o.damage;
      else if (s.damage < o.damage) f.heal[base] = o.damage - s.damage;
      if (s.id !== o.id) f.evo.add(base);
    }
    if (b[who].prizesLeft < a[who].prizesLeft) f.prize.add(who);
  }
  const ah = new Set(a.you.hand || []);
  for (const id of b.you.hand || []) if (!ah.has(id)) f.draw.add(id);
  const live = (v) => v.phase >= 2 && !v.over;
  if (live(b) && (a.turn !== b.turn || !live(a)) && b.activePlayer === ME) f.banner = "you";
  return f;
}

/* ---------------- card rendering ---------------- */
function cardEl(id, opts) {
  opts = opts || {};
  const c = cardOf(id);
  const url = imgUrl(c, opts.big);
  let el;
  if (showImages && url && !badImg.has(url)) {
    el = h("div", { class: "card img" }, h("span", { class: "ph-name", text: c.name }));
    const img = h("img", { src: url, alt: c.name, draggable: "false", loading: opts.lazy ? "lazy" : null });
    img.addEventListener("error", () => { badImg.add(url); el.replaceWith(textCard(id, opts)); });
    el.append(img);
  } else el = textCard(id, opts);
  if (!opts.noHover) wireCard(el, id);
  return el;
}
function wireCard(el, id) {
  el.addEventListener("mouseenter", () => showPreview(id));
  el.addEventListener("contextmenu", (e) => { e.preventDefault(); e.stopPropagation(); zoom(id); });
}
function textCard(id, opts) {
  const c = cardOf(id);
  const big = opts && opts.big;
  const t = (c.types && c.types[0]) || "C";
  const el = h("div", { class: "card text " + (c.super === "pokemon" ? "t-" + t : c.super) + (big ? " big" : "") });
  if (c.super === "pokemon") {
    el.append(h("div", { class: "c-row" }, h("span", { class: "c-name", style: "flex:1", text: c.name }), h("span", { class: "c-hp", text: c.hp }), eIcons(c.types)));
    el.append(h("div", { class: "c-sub", text: c.stage + (c.evolvesFrom ? " · " + c.evolvesFrom : "") }));
    for (const p of c.powers || []) {
      el.append(h("div", { class: "c-row" }, h("span", { class: "an", style: "color:#c92a2a", text: p.name })));
      if (big && p.text) el.append(h("div", { class: "c-txt", text: p.text }));
    }
    for (const a of c.attacks || []) {
      el.append(h("div", { class: "c-row" }, eIcons(a.cost), h("span", { class: "an", text: a.name }), h("b", { text: dmgText(a) })));
      if (big && a.text) el.append(h("div", { class: "c-txt", text: a.text }));
    }
    if (big) el.append(h("div", { class: "c-sub", style: "margin-top:auto" }, "W ", c.weakness.join(" ") || "–", " · R ", c.resistance.join(" ") || "–", " · ", eIcons(c.retreat)));
  } else if (c.super === "trainer") {
    el.append(h("div", { class: "c-name", text: c.name }), h("div", { class: "c-sub", text: c.trainerType }), h("div", { class: "c-txt", text: c.text || "" }));
  } else if (c.super === "energy") {
    el.append(h("div", { class: "c-row" }, h("span", { class: "c-name", style: "flex:1", text: c.name }), eIcons(c.provides)));
    if (c.energyType === "Special") el.append(h("div", { class: "c-txt", text: c.text || "" }));
  } else el.append(h("div", { class: "c-name", text: c.name }));
  return el;
}
function backEl() { return h("div", { class: "card back" }); }
let previewId = null;
function showPreview(id) {
  if (previewId === id) return;
  previewId = id;
  const p = $("#preview");
  p.replaceChildren(cardEl(id, { big: true, noHover: true }));
}
function zoom(id) {
  const z = $("#zoom");
  z.hidden = false;
  z.replaceChildren(cardEl(id, { big: true, noHover: true }));
  z.onclick = () => (z.hidden = true);
  z.oncontextmenu = (e) => { e.preventDefault(); z.hidden = true; };
}

/* ---------------- decision modes ---------------- */
function modeOf() {
  const ch = S && S.choice;
  if (!ch) return { kind: "none" };
  if (ch.type === 0) return { kind: "main" };
  const o = ch.options;
  if (!o.length) return { kind: "empty" };
  if (o.every((x) => x.kind === 1 || x.kind === 2)) return { kind: "yesno" };
  if (o.every((x) => x.kind === 0)) return { kind: "count" };
  if (ch.type === 6 || o.every((x) => x.kind === 13)) return { kind: "attack" };
  const hand = new Set(S.you.hand);
  if (o.every((x) => x.kind === 3 && x.area === 2 && x.inPlayArea === null && x.card && hand.has(x.card)) && !ch.repeats)
    return { kind: "hand" };
  const boardOK = o.every((x) => x.kind === 3 && (x.area === 4 || x.area === 5) && x.inPlayArea === null);
  if (boardOK) {
    const keys = o.map((x) => keyOf(optPlayer(x), x.area, x.index));
    if (ch.repeats || new Set(keys).size === keys.length) return { kind: "board", keys };
  }
  return { kind: "list" };
}
const single = (ch) => ch.min === 1 && ch.max === 1;
function pickCount() { return S.choice.repeats ? Object.values(ui.counts).reduce((a, b) => a + b, 0) : ui.picks.length; }
function validCount(ch, n) { return n >= ch.min && n <= ch.max; }
function chosenIndices() {
  const ch = S.choice;
  if (ch.repeats) { const out = []; for (const [i, c] of Object.entries(ui.counts)) for (let k = 0; k < c; k++) out.push(Number(i)); return out; }
  return ui.picks;
}
function togglePick(i) {
  const ch = S.choice;
  if (ch.repeats) { if (pickCount() < ch.max) ui.counts[i] = (ui.counts[i] || 0) + 1; return; }
  if (ui.picks.includes(i)) ui.picks = ui.picks.filter((x) => x !== i);
  else if (ch.max <= 1) ui.picks = [i];
  else if (ui.picks.length < ch.max) ui.picks.push(i);
}
function rangeText(ch, n) { return n + "/" + ch.max + (ch.min > 0 && ch.min < ch.max ? " · min " + ch.min : ""); }
function titleOf(ch) {
  if (promptSrc && ch.options.length && ch.options.every((o) => o.kind === 1 || o.kind === 2) && ![41, 42, 43].includes(ch.context)) return promptSrc + "?";
  if (ch.options.length && ch.options.every((o) => o.area === 6)) return "Prize";
  const t = { 1: S.you.active ? "Choose Bench" : "Choose Active", 2: "Choose Bench" }[ch.context];
  return t || ch.title || ch.contextName;
}

/* targets: slot key -> options a click on that slot answers */
function targetMap() {
  const m = new Map();
  const add = (k, o) => { if (!m.has(k)) m.set(k, []); m.get(k).push(o); };
  const md = modeOf();
  if (md.kind === "main" && ui.pending) {
    for (const o of ui.pending.opts) {
      if (o.kind === 12) add(keyOf(ME, 5, o.index), o);
      else if (o.inPlayArea) add(keyOf(optPlayer(o), o.inPlayArea, o.inPlayIndex), o);
      else if (o.kind === 7) add("bench-empty", o);
    }
  } else if (md.kind === "board") {
    S.choice.options.forEach((o, n) => add(md.keys[n], o));
  }
  return m;
}
function dragTargets(opts) {
  const keys = new Map();
  for (const o of opts) {
    const k = o.inPlayArea ? keyOf(optPlayer(o), o.inPlayArea, o.inPlayIndex) : o.kind === 7 && o.card && cardOf(o.card).super === "pokemon" ? "bench-empty" : null;
    if (k && !keys.has(k)) keys.set(k, o);
  }
  return keys;
}

/* ---------------- board ---------------- */
function slotEl(pi, area, idx, sv, targets) {
  const key = keyOf(pi, area, idx);
  if (!sv) {
    const empty = h("div", { class: "empty-slot", "data-key": pi === ME && area === 5 ? "bench-empty" : key });
    const tg = pi === ME && area === 5 ? targets.get("bench-empty") : null;
    if (tg) { empty.classList.add("target"); empty.onclick = (e) => { e.stopPropagation(); answer([tg[0].i]); }; }
    return empty;
  }
  if (sv.hidden) return h("div", { class: "slot" }, backEl());
  const base = sv.stack[0];
  const tg = targets.get(key);
  const md = modeOf();
  const wrap = h("div", { class: "slot", "data-key": key });
  const card = cardEl(sv.id);
  wrap.append(card);
  if (fx.enter.has(base)) wrap.classList.add("fx-enter");
  if (fx.evo.has(base)) wrap.classList.add("fx-evo");
  if (fx.hit[base]) { wrap.classList.add("fx-hit"); wrap.append(h("div", { class: "floater", text: "-" + fx.hit[base] })); }
  if (fx.heal[base]) wrap.append(h("div", { class: "floater heal", text: "+" + fx.heal[base] }));
  if (sv.damage) wrap.append(h("div", { class: "dmg", text: sv.damage }));
  const left = Math.max(0, sv.hp - sv.damage);
  const pct = sv.hp ? Math.round((100 * left) / sv.hp) : 0;
  wrap.append(h("div", { class: "hpbar", title: left + "/" + sv.hp }, h("i", { class: pct <= 30 ? "low" : pct <= 60 ? "mid" : "", style: "width:" + pct + "%" })));
  if (sv.energies.length) {
    const en = h("div", { class: "energies" });
    for (const e of sv.energies) {
      const c = cardOf(e);
      const ps = c.provides && c.provides.length ? c.provides : ["C"];
      const ic = h("span", { class: "e e-" + (ps.length > 1 ? "x" : ps[0]) + (c.energyType === "Special" ? " special" : ""), title: c.name, text: ps.length > 1 ? "★" : "" });
      wireCard(ic, e);
      en.append(ic);
    }
    wrap.append(en);
  }
  if (sv.tools.length) wrap.append(h("div", { class: "tools" }, sv.tools.map((t) => h("div", { class: "tool-thumb", title: cardOf(t).name }, cardEl(t)))));
  if (sv.conditions.length) wrap.append(h("div", { class: "conds" }, sv.conditions.map((c) => h("span", { class: "cond " + c, title: c, text: { Poisoned: "PSN", Burned: "BRN", Asleep: "SLP", Paralyzed: "PAR", Confused: "CNF" }[c] || c }))));
  if (sv.markers.length) wrap.append(h("div", { class: "marks" }, sv.markers.slice(0, 2).map((m) => h("span", { class: "mark", title: m, text: m }))));
  if (tg) {
    wrap.classList.add("target");
    if (md.kind === "board") {
      const o = tg[0];
      const n = S.choice.repeats ? (ui.counts[o.i] || 0) : (ui.picks.includes(o.i) ? ui.picks.indexOf(o.i) + 1 : 0);
      if (n) { wrap.classList.add("picked"); wrap.append(h("div", { class: "pick-badge", text: S.choice.repeats ? n : (S.choice.max > 1 ? n : "✓") })); }
      wrap.addEventListener("contextmenu", (e) => { if (S.choice.repeats && ui.counts[o.i]) { e.preventDefault(); e.stopPropagation(); ui.counts[o.i]--; render(); } }, true);
    }
    wrap.onclick = (e) => { e.stopPropagation(); slotClick(tg); };
  } else if (pi === ME && md.kind === "main" && !ui.pending) {
    const acts = slotActions(area, idx, sv);
    if (acts.any) {
      wrap.classList.add("click");
      if (acts.attack) wrap.classList.add("act");
      wrap.onclick = (e) => { e.stopPropagation(); openSlotPop(wrap, area, idx, sv); };
    }
  }
  return wrap;
}

function slotClick(tg) {
  const md = modeOf();
  if (md.kind === "main" && ui.pending) {
    if (tg.length === 1) return answer([tg[0].i]);
    return openChoicePop(null, "Choose", tg);
  }
  if (md.kind === "board") {
    const ch = S.choice;
    if (single(ch) && !ch.repeats) return answer([tg[0].i]);
    togglePick(tg[0].i);
    render();
  }
}

/* What a click on one of my Pokémon can do during my turn. */
function slotActions(area, idx, sv) {
  const ch = S.choice;
  const c = cardOf(sv.id);
  const out = { attacks: [], abilities: [], retreat: [], any: false, attack: false };
  for (const o of ch.options) {
    if (o.kind === 13 && area === 4) out.attacks.push(o);
    else if (o.kind === 10 && o.area === area && (area === 4 || o.index === idx)) out.abilities.push(o);
    else if (o.kind === 12 && area === 4) out.retreat.push(o);
  }
  out.attack = out.attacks.length > 0;
  out.any = out.attack || out.abilities.length > 0 || out.retreat.length > 0 || area === 4;
  return out;
}

function openSlotPop(anchor, area, idx, sv) {
  const acts = slotActions(area, idx, sv);
  const c = cardOf(sv.id);
  const pop = h("div", { class: "menu-list" }, h("div", { class: "pop-h", text: c.name }));
  const powers = (c.powers || []);
  acts.abilities.forEach((o, n) => {
    const p = powers[n] || powers[0];
    pop.append(h("button", { class: "act", title: p ? p.text : "", onclick: () => answer([o.i]) }, h("span", { class: "kind", text: "Ability" }), h("span", { class: "nm", text: p ? p.name : o.label })));
  });
  // Printed abilities that cannot be used right now (shown greyed for reference).
  for (const p of powers.slice(acts.abilities.length)) {
    pop.append(h("button", { class: "act", title: p.text || "", disabled: true }, h("span", { class: "kind", text: "Ability" }), h("span", { class: "nm", text: p.name })));
  }
  if (area === 4) {
    const used = new Set();
    (c.attacks || []).forEach((a, ai) => {
      const o = acts.attacks.find((x) => x.attackId === ai && (!x.card || x.card === sv.id) && !used.has(x.i));
      if (o) used.add(o.i);
      pop.append(h("button", { class: "act", title: a.text || "", disabled: !o, onclick: () => o && answer([o.i]) },
        h("span", { class: "costs" }, eIcons(a.cost.length ? a.cost : [])), h("span", { class: "nm", text: a.name }), h("span", { class: "dm", text: dmgText(a) })));
    });
    for (const o of acts.attacks) {
      if (used.has(o.i)) continue;
      const src = o.card ? cardOf(o.card) : null;
      const a = src && src.attacks && src.attacks[o.attackId];
      pop.append(h("button", { class: "act", title: a ? a.text : "", onclick: () => answer([o.i]) },
        h("span", { class: "costs" }, eIcons(a ? a.cost : [])), h("span", { class: "nm", text: a ? a.name : o.label }), h("span", { class: "dm", text: dmgText(a) })));
    }
    pop.append(h("button", { class: "act", disabled: !acts.retreat.length, onclick: () => { closePop(); ui.pending = { label: "Retreat", opts: acts.retreat, src: sv.id }; render(); } },
      h("span", { class: "nm", text: "Retreat" }), h("span", { class: "costs" }, eIcons(c.retreat))));
  }
  showPop(anchor, pop);
}
function openChoicePop(anchor, title, opts) {
  const pop = h("div", { class: "menu-list" }, h("div", { class: "pop-h", text: title }),
    opts.map((o) => h("button", { class: "act", onclick: () => answer([o.i]) }, h("span", { class: "nm", text: o.label }))));
  showPop(anchor, pop);
}
function showPop(anchor, content) {
  const p = $("#pop");
  p.replaceChildren(content);
  p.hidden = false;
  const r = anchor ? anchor.getBoundingClientRect() : { left: innerWidth / 2 - 110, right: innerWidth / 2 - 110, top: innerHeight / 2 - 80, bottom: innerHeight / 2 };
  const pw = p.offsetWidth, ph = p.offsetHeight;
  let x = r.right + 10, y = r.top;
  if (x + pw > innerWidth - 8) x = Math.max(8, r.left - pw - 10);
  if (y + ph > innerHeight - 8) y = innerHeight - ph - 8;
  p.style.left = x + "px"; p.style.top = Math.max(8, y) + "px";
  ui.pop = true;
}
function closePop() { const p = $("#pop"); p.hidden = true; p.replaceChildren(); ui.pop = null; }

/* drag a hand card onto a highlighted target (pointer events, so it also works with touch) */
let drag = null;
let lastDrag = 0;
function wireHandDrag(wrap, id, targets) {
  wrap.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || busy) return;
    const sx = e.clientX, sy = e.clientY;
    let ghost = null;
    const over = (ev) => { const el = document.elementFromPoint(ev.clientX, ev.clientY); return el && el.closest("[data-key]"); };
    const move = (ev) => {
      if (!ghost) {
        if (Math.hypot(ev.clientX - sx, ev.clientY - sy) < 8) return;
        closePop();
        ghost = wrap.querySelector(".card").cloneNode(true);
        ghost.classList.add("ghost");
        document.body.append(ghost);
        wrap.classList.add("dragging");
        drag = { id, targets };
        for (const k of targets.keys()) document.querySelectorAll('[data-key="' + k + '"]').forEach((el) => el.classList.add("drag-target", "target"));
      }
      ghost.style.left = ev.clientX + "px"; ghost.style.top = ev.clientY + "px";
      const o = over(ev);
      document.querySelectorAll(".dragover").forEach((x) => { if (x !== o) x.classList.remove("dragover"); });
      if (o && targets.has(o.dataset.key)) o.classList.add("dragover");
    };
    const up = (ev) => {
      removeEventListener("pointermove", move); removeEventListener("pointerup", up);
      if (!ghost) return;
      ghost.remove(); wrap.classList.remove("dragging");
      const el = over(ev);
      const o = el && targets.get(el.dataset.key);
      endDrag();
      lastDrag = Date.now();
      if (o) answer([o.i]);
    };
    addEventListener("pointermove", move); addEventListener("pointerup", up);
  });
}
function endDrag() {
  drag = null;
  document.querySelectorAll(".dragover,.drag-target").forEach((x) => x.classList.remove("dragover", "drag-target", "target"));
}

function prizesEl(pv, who) {
  const box = h("div", { class: "prizes" + (fx.prize.has(who) ? " fx-prize" : "") });
  for (const p of pv.prizes) {
    if (p.state === "taken") box.append(h("div", { class: "prize taken" }));
    else if (p.state === "up") box.append(cardEl(p.id, { noHover: false }));
    else box.append(h("div", { class: "card back prize" }));
  }
  box.querySelectorAll(".card").forEach((c) => { c.style.width = "100%"; c.style.height = "100%"; c.style.borderWidth = "2px"; });
  box.append(h("div", { class: "zone-count", title: "Prizes", text: pv.prizesLeft }));
  return box;
}
function pilesEl(pv, who) {
  const deck = h("div", { class: "pile" }, pv.deckCount ? backEl() : h("div", { class: "pile empty", text: "Deck" }), h("div", { class: "zone-count", text: pv.deckCount }));
  const top = pv.discard[pv.discard.length - 1];
  const disc = h("div", { class: "pile" + (pv.discard.length ? " click" : "") }, top ? cardEl(top) : h("div", { class: "pile empty", text: "Discard" }));
  if (pv.discard.length) {
    disc.append(h("div", { class: "zone-count", style: "background:#495057", text: pv.discard.length }));
    disc.onclick = () => viewPile((who === "you" ? "Your" : "Opponent") + " discard", pv.discard);
  }
  if (pv.lostzone.length) {
    const lz = h("span", { class: "lz", text: "LZ " + pv.lostzone.length, style: "cursor:pointer" });
    lz.onclick = (e) => { e.stopPropagation(); viewPile("Lost Zone", pv.lostzone); };
    disc.append(lz);
  }
  return h("div", { class: "piles" }, who === "you" ? [deck, disc] : [disc, deck]);
}
function viewPile(title, ids) {
  ui.modal = { kind: "pile", title, ids: [...ids].reverse() };
  renderModal();
}
function rowEl(pv, who, targets) {
  const bn = Math.max(5, pv.bench.length);
  const bench = h("div", { class: "bench", style: "--bn:" + bn },
    pv.bench.map((s, i) => slotEl(pv.index, 5, i, s, targets)));
  for (let i = pv.bench.length; i < 5; i++) bench.append(h("div", { class: "empty-slot" }));
  return [prizesEl(pv, who), bench, pilesEl(pv, who)];
}

function renderBoard() {
  const targets = targetMap();
  $("#opp-row").replaceChildren(...rowEl(S.opp, "opp", targets));
  $("#you-row").replaceChildren(...rowEl(S.you, "you", targets));
  $("#opp-active").replaceChildren(slotEl(OPP, 4, 0, S.opp.active, targets));
  $("#you-active").replaceChildren(slotEl(ME, 4, 0, S.you.active, targets));
  // opponent hand
  const oh = $("#opp-hand");
  oh.replaceChildren();
  for (let i = 0; i < Math.min(S.opp.handCount, 14); i++) oh.append(backEl());
  oh.append(h("span", { class: "n", text: S.opp.handCount }));
  // left column: tags + stadium
  const live = S.phase >= 2 && !S.over;
  const tag = (who, pv) => {
    const mine = live && S.activePlayer === (who === "you" ? ME : OPP);
    const t = h("div", {}, h("div", { class: "tag " + who + (mine ? " turn" : "") }, h("span", { class: "dot" }), who === "you" ? "You" : "Opponent"));
    if (who === "you" && live) {
      const fl = (txt, used) => h("span", { class: "flag" + (used ? " used" : ""), text: txt });
      t.append(h("div", { class: "flags" }, fl("Energy", mine && pv.energyAttached), fl("Supporter", mine && pv.supporterPlayed), fl("Retreat", mine && pv.retreated)));
    }
    return t;
  };
  const left = $("#c-left");
  left.replaceChildren(tag("opp", S.opp));
  const mid = h("div", { style: "display:flex;gap:14px;align-items:center;align-self:center" });
  for (const [pv, who] of [[S.opp, "Opp"], [S.you, "You"]]) {
    if (pv.supporter.length) mid.append(h("div", { class: "stadium-zone sup-zone" }, h("span", { class: "slot-lbl", text: (who === "You" ? "" : "Opp. ") + (cardOf(pv.supporter[0]).trainerType || "Trainer") }), cardEl(pv.supporter[0])));
  }
  if (S.stadium) {
    const z = h("div", { class: "stadium-zone " + (S.stadium.owner === ME ? "yours" : "theirs") }, h("span", { class: "slot-lbl", text: "Stadium" }));
    const sc = cardEl(S.stadium.id);
    const use = modeOf().kind === "main" && !ui.pending ? S.choice.options.filter((o) => o.kind === 15 && o.area === 7) : [];
    if (use.length) { sc.classList.add("click"); sc.onclick = (e) => { e.stopPropagation(); answer([use[0].i]); }; sc.title = "Use Stadium"; }
    z.append(sc);
    mid.append(z);
  }
  left.append(mid, tag("you", S.you));
  $("#board").onclick = () => { if (ui.pending) { ui.pending = null; render(); } };
}

/* ---------------- hand ---------------- */
function setupBadge(ch, n) {
  if (ch.context !== 1 && ch.context !== 2) return n + 1;
  if (ch.context === 1 && !S.you.active) return n === 0 ? "A" : "B" + n;
  return "B" + (n + 1);
}
function renderHand() {
  const hand = $("#hand");
  hand.replaceChildren();
  const md = modeOf();
  const ch = S.choice;
  const sorted = [...S.you.hand];
  const order = { pokemon: 0, trainer: 1, energy: 2 };
  sorted.sort((a, b) => { const A = cardOf(a), B = cardOf(b); return ((order[A.super] ?? 3) - (order[B.super] ?? 3)) || A.name.localeCompare(B.name); });
  for (const id of sorted) {
    const wrap = h("div", { class: "hc" + (fx.draw.has(id) ? " fx-draw" : "") });
    wrap.append(cardEl(id));
    if (md.kind === "main") {
      const opts = ch.options.filter((o) => o.area === 2 && o.card === id && o.kind !== 10);
      if (opts.length) {
        wrap.classList.add("playable");
        if (ui.pending && ui.pending.cardId === id) wrap.classList.add("pending");
        else if (ui.pending) wrap.classList.add("dim");
        wrap.onclick = (e) => { e.stopPropagation(); handClick(wrap, id, opts); };
        const targets = dragTargets(opts);
        if (targets.size) wireHandDrag(wrap, id, targets);
      } else if (ui.pending) wrap.classList.add("dim");
    } else if (md.kind === "hand") {
      const o = ch.options.find((x) => x.card === id);
      if (o) {
        const n = ui.picks.indexOf(o.i);
        wrap.classList.add(n >= 0 ? "picked" : "pickable");
        if (n >= 0 && ch.max > 1) wrap.append(h("div", { class: "pick-badge", text: setupBadge(ch, n) }));
        wrap.onclick = (e) => { e.stopPropagation(); if (single(ch)) return answer([o.i]); togglePick(o.i); render(); };
      } else wrap.classList.add("dim");
    }
    hand.append(wrap);
  }
  requestAnimationFrame(() => { fitRows(); fitHand(); });
}
/* Shrink bench cards when a big bench (e.g. 8 slots) would not fit the board width. */
function fitRows() {
  const W = $("#board").clientWidth - 32;
  for (const row of [$("#opp-row"), $("#you-row")]) {
    const bench = row.querySelector(".bench"), pile = row.querySelector(".pile");
    if (!bench || !pile) continue;
    const bn = Number(bench.style.getPropertyValue("--bn")) || 5;
    const other = row.scrollWidth - bench.offsetWidth;
    const cw = pile.offsetWidth;
    const bs = Math.min(1, (W - other - 18 - (bn - 1) * 8) / (bn * cw));
    bench.style.setProperty("--bs", Math.max(0.4, bs).toFixed(3));
  }
}
function fitHand() {
  const hand = $("#hand");
  const kids = [...hand.querySelectorAll(".hc")];
  if (!kids.length) return;
  const w = kids[0].offsetWidth;
  const avail = hand.clientWidth - 16;
  const gap = 6;
  const need = kids.length * w + (kids.length - 1) * gap;
  const m = need > avail ? -((kids.length * w - avail) / (kids.length - 1)) : gap;
  kids.forEach((k, i) => (k.style.marginLeft = i ? m + "px" : "0"));
}
function handClick(el, id, opts) {
  if (Date.now() - lastDrag < 300) return;
  closePop();
  if (ui.pending && ui.pending.cardId === id) { ui.pending = null; return render(); }
  const targeted = opts.filter((o) => o.inPlayArea || o.kind === 7 && cardOf(id).super === "pokemon");
  if (opts.length === 1 && !opts[0].inPlayArea) return answer([opts[0].i]);
  if (targeted.length === opts.length) {
    if (opts.length === 1) return answer([opts[0].i]);
    ui.pending = { cardId: id, label: cardOf(id).name, opts, src: id };
    return render();
  }
  openChoicePop(el, cardOf(id).name, opts);
}

/* ---------------- control panel ---------------- */
function btn(label, cls, onclick, disabled) { return h("button", { class: "btn " + (cls || ""), onclick, disabled: !!disabled }, label); }
function promptBox(title, cnt, ...rows) {
  const src = promptSrc && S.choice && S.choice.type !== 0 ? h("div", { class: "psrc", text: promptSrc }) : null;
  return h("div", { class: "prompt" }, src, h("div", { class: "pt" }, h("span", { text: title }), cnt !== null && cnt !== undefined ? h("span", { class: "cnt", text: cnt }) : null), ...rows);
}
function renderCtrl() {
  const c = $("#ctrl");
  c.replaceChildren();
  const md = modeOf();
  const ch = S.choice;
  if (S.over) {
    c.append(promptBox(S.winner === 0 ? "Victory" : S.winner === 1 ? "Defeat" : "Game over", null,
      h("div", { class: "pr" }, btn("Rematch", "primary", rematch), btn("New game", "", () => openNewGame(false)))));
    return;
  }
  if (!ch) { c.append(promptBox(S.error ? "Error" : "Waiting", null, S.error ? h("div", { class: "err", text: S.error }) : null)); return; }
  const live = S.phase >= 2;
  if (md.kind === "main") {
    c.append(h("span", { class: "turn-pill you", text: "Your turn" }));
    if (ui.pending) {
      c.append(promptBox(ui.pending.label, null,
        h("div", { class: "pr" }, btn("Cancel", "", () => { ui.pending = null; render(); }))));
      return;
    }
    const extra = ch.options.filter((o) => o.kind === 15 && o.area !== 7 || o.kind === 16 || o.kind === 11);
    const row = h("div", { class: "pr" });
    for (const o of extra) row.append(btn(o.label, "", () => answer([o.i])));
    const canAttack = ch.options.some((o) => o.kind === 13);
    const end = ch.options.find((o) => o.kind === 14);
    if (end) {
      const armed = ui.endArm;
      c.append(btn(armed ? "Confirm end" : "End turn", "big end-btn " + (armed ? "danger" : "primary"), () => {
        if (canAttack && !ui.endArm) { ui.endArm = true; render(); clearTimeout(renderCtrl.t); renderCtrl.t = setTimeout(() => { ui.endArm = false; if (S) renderCtrl(); }, 2500); return; }
        answer([end.i]);
      }));
    }
    row.append(h("button", { class: "btn", title: "All legal actions", onclick: (e) => { e.stopPropagation(); openChoicePop(e.currentTarget, "Actions", ch.options); } }, "⋯"));
    c.append(row);
    return;
  }
  if (live) c.append(h("span", { class: "turn-pill " + (S.activePlayer === ME ? "you" : "opp"), text: S.activePlayer === ME ? "Your turn" : "Opponent's turn" }));
  const title = titleOf(ch);
  const err = ui.err ? h("div", { class: "err", text: ui.err }) : null;
  if (md.kind === "board" || md.kind === "hand") {
    const n = pickCount();
    const auto = single(ch) && !ch.repeats;
    c.append(promptBox(title, auto ? null : rangeText(ch, n), err, auto ? null : h("div", { class: "pr" },
      btn("Confirm", "primary", () => answer(chosenIndices()), !validCount(ch, n)),
      n ? btn("Clear", "", () => { ui.picks = []; ui.counts = {}; render(); }) : null)));
  } else if (md.kind === "yesno") {
    c.append(promptBox(title, null, err, h("div", { class: "pr" }, ch.options.map((o) => btn(o.kind === 1 ? "Yes" : "No", o.kind === 1 ? "primary" : "", () => answer([o.i]))))));
  } else if (md.kind === "count") {
    c.append(promptBox(title, null, err, h("div", { class: "pr" }, ch.options.map((o) => btn(String(o.number), "", () => answer([o.i]))))));
  } else if (md.kind === "empty") {
    c.append(promptBox(title, "0", err, h("div", { class: "pr" }, btn("OK", "primary", () => answer([])))));
  } else {
    c.append(promptBox(title, ui.minimized ? null : rangeText(ch, pickCount()), err,
      ui.minimized ? h("div", { class: "pr" }, btn("Show", "primary", () => { ui.minimized = false; render(); })) : null));
  }
}

/* ---------------- modals ---------------- */
function renderModal() {
  const m = $("#modal");
  const md = S ? modeOf() : { kind: "none" };
  m.onclick = null;
  if (ui.modal && ui.modal.kind === "pile") {
    const { title, ids } = ui.modal;
    const close = () => { ui.modal = null; renderModal(); };
    m.hidden = false;
    m.replaceChildren(h("div", { class: "mbox" },
      h("header", {}, h("h2", { text: title }), h("span", { class: "cnt", text: ids.length }), h("span", { class: "sp" }), btn("Close", "", close)),
      h("div", { class: "mb" }, h("div", { class: "grid" }, ids.map((id) => h("div", { class: "tile" }, cardEl(id, { lazy: true })))))));
    m.onclick = (e) => { if (e.target === m) close(); };
    return;
  }
  if (S && S.over && !ui.overDismissed) return renderGameOver(m);
  if (!S || !S.choice || ui.minimized || !(md.kind === "list" || md.kind === "attack")) { m.hidden = true; m.replaceChildren(); return; }
  const ch = S.choice;
  m.hidden = false;
  const hide = btn("Board", "", () => { ui.minimized = true; render(); });
  if (md.kind === "attack") {
    const list = h("div", { class: "menu-list" }, ch.options.map((o) => {
      const c = o.card ? cardOf(o.card) : null;
      const a = c && c.attacks && c.attacks[o.attackId];
      return h("button", { class: "act", onclick: () => answer([o.i]) }, h("span", { class: "costs" }, eIcons(a ? a.cost : [])),
        h("span", { class: "nm", text: a ? a.name : o.label }), c ? h("span", { class: "cnt", style: "color:var(--dim);font-size:11px", text: c.name }) : null, h("span", { class: "dm", text: dmgText(a) }));
    }));
    m.replaceChildren(h("div", { class: "mbox" }, h("header", {}, promptSrc ? h("span", { class: "psrc", text: promptSrc }) : null, h("h2", { text: titleOf(ch) }), h("span", { class: "sp" }), hide), h("div", { class: "mb" }, list),
      ui.err ? h("footer", {}, h("span", { class: "err", text: ui.err })) : null));
    return;
  }
  const grid = h("div", { class: "grid" });
  const cnt = h("span", { class: "cnt" });
  const multiHolder = new Set(ch.options.map((o) => o.card && holderOf(o.card)).filter(Boolean)).size > 1;
  const confirm = btn("Confirm", "primary", () => answer(chosenIndices()));
  const clear = btn("Clear", "", () => { ui.picks = []; ui.counts = {}; paint(); });
  // Identical cards (same printing, same caption) collapse into one tile with a count,
  // unless pick order matters (putting cards on the deck in order).
  const groups = [];
  if (!ch.repeats && ![9, 10].includes(ch.context)) {
    const by = new Map();
    for (const o of ch.options) {
      const k = o.card ? o.card.split("#")[0] + "|" + capOf(o, multiHolder) : "#" + o.i;
      if (!by.has(k)) { by.set(k, []); groups.push(by.get(k)); }
      by.get(k).push(o);
    }
  }
  const grouped = groups.length && groups.length < ch.options.length;
  const paintGroups = () => {
    grid.replaceChildren();
    for (const g of groups) {
      const o = g[0];
      const nSel = g.filter((x) => ui.picks.includes(x.i)).length;
      const t = h("div", { class: "tile" + (nSel ? " sel" : "") + (o.card || o.area === 6 ? "" : " wide") });
      t.onclick = () => {
        if (single(ch)) return answer([o.i]);
        const free = g.find((x) => !ui.picks.includes(x.i));
        if (free && (ch.max <= 1 || ui.picks.length < ch.max)) togglePick(free.i);
        else { const last = [...g].reverse().find((x) => ui.picks.includes(x.i)); if (last) togglePick(last.i); }
        paintGroups();
      };
      t.oncontextmenu = (e) => { e.preventDefault(); const last = [...g].reverse().find((x) => ui.picks.includes(x.i)); if (last) { togglePick(last.i); paintGroups(); } };
      if (o.card) t.append(cardEl(o.card, { noHover: false }));
      else if (o.area === 6) t.append(backEl());
      else t.append(h("div", { class: "box", text: o.label }));
      if (g.length > 1) t.append(h("div", { class: "grp-n", text: "×" + g.length }));
      t.append(h("div", { class: "cap", text: capOf(o, multiHolder) }));
      if (nSel) t.append(h("div", { class: "pick-badge", text: g.length > 1 ? nSel + "/" + g.length : "✓" }));
      grid.append(t);
    }
    const n = pickCount();
    cnt.textContent = rangeText(ch, n);
    confirm.disabled = !validCount(ch, n);
    clear.disabled = !n;
  };
  const paint = () => {
    if (grouped) return paintGroups();
    grid.replaceChildren();
    for (const o of ch.options) {
      const sel = ui.picks.includes(o.i) || (ui.counts[o.i] || 0) > 0;
      const t = h("div", { class: "tile" + (sel ? " sel" : "") + (o.card || o.area === 6 ? "" : " wide") });
      t.onclick = () => { if (ch.repeats) return; if (single(ch)) return answer([o.i]); togglePick(o.i); paint(); };
      if (o.card) t.append(cardEl(o.card));
      else if (o.area === 6) t.append(backEl());
      else t.append(h("div", { class: "box", text: o.label }));
      t.append(h("div", { class: "cap", text: capOf(o, multiHolder) }));
      if (sel && !ch.repeats && ch.max > 1) t.append(h("div", { class: "pick-badge", text: ui.picks.indexOf(o.i) + 1 }));
      if (ch.repeats) {
        t.append(h("div", { class: "ctr" },
          h("button", { onclick: (e) => { e.stopPropagation(); if (ui.counts[o.i]) { ui.counts[o.i]--; paint(); } } }, "−"),
          h("b", { text: ui.counts[o.i] || 0 }),
          h("button", { onclick: (e) => { e.stopPropagation(); if (pickCount() < ch.max) { ui.counts[o.i] = (ui.counts[o.i] || 0) + 1; paint(); } } }, "+")));
      }
      grid.append(t);
    }
    const n = pickCount();
    cnt.textContent = rangeText(ch, n);
    confirm.disabled = !validCount(ch, n);
    clear.disabled = !n;
  };
  paint();
  m.replaceChildren(h("div", { class: "mbox" },
    h("header", {}, promptSrc ? h("span", { class: "psrc", text: promptSrc }) : null, h("h2", { text: titleOf(ch) }), cnt, h("span", { class: "sp" }), hide),
    h("div", { class: "mb" }, grid),
    h("footer", {}, h("span", { class: "err", text: ui.err }), h("span", { class: "sp" }), clear, confirm)));
}
/* Short caption under a choice tile: where it goes / what it is attached to. */
function holderOf(id) {
  for (const pv of [S.you, S.opp]) for (const s of [pv.active, ...pv.bench]) {
    if (s && !s.hidden && (s.energies.includes(id) || s.tools.includes(id))) return cardOf(s.id).name;
  }
  return null;
}
function capOf(o, multiHolder) {
  if (multiHolder && o.card) { const hn = holderOf(o.card); if (hn) return "on " + hn; }
  if (o.area === 6 && !o.card) return "Prize " + (o.index + 1);
  if (o.inPlayArea) {
    const pv = optPlayer(o) === ME ? S.you : S.opp;
    const s = topSlot(pv, o.inPlayArea, o.inPlayIndex);
    return "→ " + (s && s.id ? cardOf(s.id).name : o.inPlayArea === 4 ? "Active" : "Bench");
  }
  if (o.card && (o.area === 4 || o.area === 5)) {
    const pv = optPlayer(o) === ME ? S.you : S.opp;
    const s = topSlot(pv, o.area, o.index);
    if (s && s.id && s.id !== o.card) return "on " + cardOf(s.id).name;
  }
  return "";
}

/* Best guess at why the game ended (the binding reports only the winner). */
function overReason() {
  const w = S.winner;
  if (w !== 0 && w !== 1) return "";
  const [win, lose] = w === 0 ? [S.you, S.opp] : [S.opp, S.you];
  if (win.prizesLeft === 0) return "All Prizes taken";
  if (!lose.active && lose.bench.every((b) => !b)) return "No Pokémon left in play";
  if (lose.deckCount === 0) return "Deck out";
  return "";
}
function renderGameOver(m) {
  const w = S.winner;
  const you = 6 - S.you.prizesLeft, opp = 6 - S.opp.prizesLeft;
  m.hidden = false;
  m.replaceChildren(h("div", { class: "mbox" },
    h("div", { class: "over" },
      h("h1", { class: w === 0 ? "win" : w === 1 ? "lose" : "", text: w === 0 ? "Victory" : w === 1 ? "Defeat" : "Game over" }),
      h("div", { class: "why", text: overReason() }),
      h("div", { class: "sub", text: S.deckNames[0] + " vs " + S.deckNames[1] }),
      h("div", { class: "score" },
        h("div", {}, h("b", { text: you }), h("span", { text: "Your prizes" })),
        h("div", {}, h("b", { text: S.turn }), h("span", { text: "Turns" })),
        h("div", {}, h("b", { text: opp }), h("span", { text: "Opp. prizes" }))),
      S.error ? h("div", { class: "sub", style: "color:#ffa8a8", text: S.error }) : null),
    h("footer", { style: "justify-content:center" }, btn("Board", "", () => { ui.overDismissed = true; render(); }), btn("New game", "", () => openNewGame(false)), btn("Rematch", "primary big", rematch))));
}

/* ---------------- new game ---------------- */
let lastDecks = {};
try { lastDecks = JSON.parse(localStorage.getItem("ptcg.decks") || "{}"); } catch (e) { /* ignore */ }
async function startGame(a, b, seed, bot) {
  try { localStorage.setItem("ptcg.decks", JSON.stringify({ a, b, bot })); } catch (e) { /* ignore */ }
  lastDecks = { a, b, bot };
  const r = await api("/api/new", { deckA: a, deckB: b, seed, bot });
  if (!r.ok) return toast(r.body.error || "Failed");
  ui = fresh(); previewId = null; $("#preview").replaceChildren(h("div", { class: "ph" }));
  applyView(r.body, true); render();
}
function rematch() { if (S) startGame(S.decks[0], S.decks[1], undefined, lastDecks.bot); }
async function openNewGame(first) {
  closePop();
  const d = (await api("/api/decks")).body;
  const m = $("#modal");
  const mk = (sel) => h("select", {}, d.decks.map((x) => h("option", { value: x.id, selected: x.id === sel }, x.name)));
  const a = mk(lastDecks.a || "dragapult-ex"), b = mk(lastDecks.b || "raging-bolt-ex");
  const seed = h("input", { type: "number", placeholder: "random", style: "width:110px" });
  const bot = h("select", { style: "width:auto" }, d.bots.map((x) => h("option", { value: x, text: x, selected: x === lastDecks.bot })));
  const la = h("div", { class: "dl" }), lb = h("div", { class: "dl" });
  const fill = async (sel, box) => {
    const r = await api("/api/deck?id=" + encodeURIComponent(sel.value));
    const rows = r.body.cards || [];
    for (const row of rows) if (row.card) cards[row.card.ref] = row.card;
    box.replaceChildren(...rows.map((row) => h("div", { class: "di", title: row.count + " " + row.name }, row.card ? cardEl(row.card.ref, { lazy: true }) : h("div", { class: "card text", text: row.name }), h("b", { text: row.count }))));
  };
  a.onchange = () => fill(a, la); b.onchange = () => fill(b, lb);
  fill(a, la); fill(b, lb);
  const close = () => { ui.modal = null; render(); };
  m.hidden = false;
  m.onclick = null;
  m.replaceChildren(h("div", { class: "mbox", style: "width:min(980px,94vw)" },
    h("header", {}, h("h2", { text: "New game" }), h("span", { class: "sp" }), first || !S ? null : btn("Cancel", "", close)),
    h("div", { class: "mb" }, h("div", { class: "ng" },
      h("div", { class: "col" }, h("div", { class: "lbl" }, h("span", { class: "tag you" }, h("span", { class: "dot" })), "You"), a, la),
      h("div", { class: "col" }, h("div", { class: "lbl" }, h("span", { class: "tag opp" }, h("span", { class: "dot" })), "Opponent"), b, lb))),
    h("footer", {}, h("span", { class: "lbl", style: "color:var(--dim)" }, "Seed"), seed, h("span", { style: "color:var(--dim)" }, "Bot"), bot, h("span", { class: "sp" }),
      btn("Start", "primary big", () => startGame(a.value, b.value, seed.value === "" ? undefined : Number(seed.value), bot.value)))));
  ui.modal = { kind: "new" };
}

/* ---------------- side panel ---------------- */
function renderSide() {
  const st = $("#status");
  const live = S.phase >= 2 && !S.over;
  st.replaceChildren(...[
    h("span", { class: "t", text: S.over ? "End" : live ? "T" + S.turn : "Setup" }),
    live ? h("span", { class: "turn-pill " + (S.activePlayer === ME ? "you" : "opp"), text: S.activePlayer === ME ? "You" : "Opp" }) : null,
    h("span", { class: "vs", title: "seed " + S.seed, text: S.deckNames[0] + " vs " + S.deckNames[1] })].filter(Boolean));
  if (!previewId) {
    const a = S.you.active && !S.you.active.hidden ? S.you.active.id : null;
    $("#preview").replaceChildren(a ? cardEl(a, { big: true, noHover: true }) : h("div", { class: "ph" }));
  }
  const el = $("#log");
  const stick = el.scrollTop + el.clientHeight >= el.scrollHeight - 40;
  el.replaceChildren(...logLines.filter((l) => l.kind !== "info" || l.who !== "sys").map((l) => h("div", { class: "ll " + l.who + " k-" + (l.kind || "act"), text: l.text })));
  if (stick) el.scrollTop = el.scrollHeight;
}
function openMenu(anchor) {
  const pop = h("div", { class: "menu-list" },
    h("button", { class: "act", onclick: () => { closePop(); openNewGame(!S); } }, h("span", { class: "nm", text: "New game" })),
    S ? h("button", { class: "act", onclick: () => { closePop(); rematch(); } }, h("span", { class: "nm", text: "Rematch" })) : null,
    h("button", { class: "act", onclick: () => { showImages = !showImages; try { localStorage.setItem("ptcg.images", showImages ? "1" : "0"); } catch (e) { /* ignore */ } closePop(); previewId = null; if (S) render(); } },
      h("span", { class: "nm", text: "Card images" }), h("span", { text: showImages ? "On" : "Off" })),
    S ? h("div", { class: "pop-h", text: "Seed " + S.seed }) : null);
  showPop(anchor, pop);
  const p = $("#pop"); const r = anchor.getBoundingClientRect();
  p.style.left = Math.max(8, r.right - p.offsetWidth) + "px"; p.style.top = r.bottom + 6 + "px";
}

/* ---------------- top level ---------------- */
function render() {
  if (!S) return;
  if (ui.modal && ui.modal.kind === "new") ui.modal = null;
  renderBoard(); renderHand(); renderCtrl(); renderSide(); renderModal();
  if (fx.banner) {
    const b = $("#banner");
    b.className = fx.banner; b.textContent = fx.banner === "you" ? "Your turn" : "Opponent's turn";
    b.hidden = false; b.style.animation = "none"; void b.offsetWidth; b.style.animation = "";
    clearTimeout(render.bt); render.bt = setTimeout(() => (b.hidden = true), 1000);
  }
  fx = noFx();
}

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (!$("#zoom").hidden) $("#zoom").hidden = true;
    else if (ui.pop) closePop();
    else if (ui.modal && ui.modal.kind === "pile") { ui.modal = null; renderModal(); }
    else if (ui.pending) { ui.pending = null; render(); }
  } else if (e.key === "Enter" && S && S.choice && !e.target.matches("input,select")) {
    const b = [...document.querySelectorAll("#modal .btn.primary, #ctrl .prompt .btn.primary")].find((x) => !x.disabled && x.textContent === "Confirm");
    if (b) b.click();
  }
});
document.addEventListener("click", (e) => { if (ui.pop && !$("#pop").contains(e.target)) closePop(); });
$("#btn-menu").addEventListener("click", (e) => { e.stopPropagation(); openMenu(e.currentTarget); });
window.addEventListener("resize", () => { if (S) { fitRows(); fitHand(); } });
boot();
