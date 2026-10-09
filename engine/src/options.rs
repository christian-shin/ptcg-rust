//! Turn-level options: structural candidates, each answered from the declared
//! checks (`legal.rs`); a trial on a copy of the game only for what is not
//! declared (a counted fallback). Mirrors the oracle's `turnCandidates` +
//! `legalTurnOptions` exactly, including order.

use crate::game::{Action, Game};
use crate::legal::{legal_fast, Ctx};
use crate::list::*;
use crate::prompts::{slot_targets, target_json};
use crate::types::*;
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct TurnOption {
    pub desc: Value,
    pub action: Action,
}

const BOARD: CardTarget = CardTarget::new(PlayerType::BottomPlayer, SlotType::Board, 0);

/// JavaScript default sort order for strings (UTF-16 code units).
fn js_sort(v: &mut Vec<&'static str>) {
    v.sort_by(|a, b| {
        let x: Vec<u16> = a.encode_utf16().collect();
        let y: Vec<u16> = b.encode_utf16().collect();
        x.cmp(&y)
    });
}

/// Canonical descriptor of a turn action (oracle `TurnOption.desc`).
pub fn describe_action(g: &Game, a: Action) -> Value {
    let p = g.st.active_player as usize;
    match a {
        Action::PlayCard { hand_index, target } => {
            let c = g.st.players[p].hand.as_slice()[hand_index as usize];
            json!({ "a": "play", "card": g.card_ref(c), "target": target_json(target) })
        }
        Action::Attack { name, from: None } => json!({ "a": "attack", "name": name }),
        Action::Attack { name, from: Some(f) } => json!({ "a": "attack", "name": name, "from": f }),
        Action::UseAbility { name, target } => json!({ "a": "ability", "name": name, "source": target_json(target) }),
        Action::UseTrainerAbility { name, target } => json!({ "a": "trainerAbility", "name": name, "source": target_json(target) }),
        Action::UseStadium => json!({ "a": "stadium" }),
        Action::Retreat { bench_index } => json!({ "a": "retreat", "bench": bench_index }),
        Action::Pass => json!({ "a": "pass" }),
    }
}

pub fn turn_candidates(g: &Game) -> Vec<TurnOption> {
    candidate_actions(g).into_iter().map(|a| TurnOption { desc: describe_action(g, a), action: a }).collect()
}

/// Structural candidates for the active player's turn, in oracle order.
pub fn candidate_actions(g: &Game) -> Vec<Action> {
    candidates(&mut Ctx::new(g))
}

/// The candidates, reading the attack list through the decision's context.
fn candidates(ctx: &mut Ctx) -> Vec<Action> {
    let g = ctx.g;
    let mut out = Vec::with_capacity(32);
    let p = g.st.active_player as usize;
    let own = slot_targets(&g.st, p, PlayerType::BottomPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]);
    let pl = &g.st.players[p];
    let first_empty = pl.bench.iter().position(|b| pl.slots[*b as usize].cards.is_empty());
    for (hi, c) in pl.hand.iter().enumerate() {
        let d = g.st.cdef(c);
        let hand_index = hi as u8;
        if d.is_energy() {
            for t in &own {
                out.push(Action::PlayCard { hand_index, target: *t });
            }
        } else if d.is_pokemon() {
            if let Some(i) = first_empty {
                out.push(Action::PlayCard { hand_index, target: CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8) });
            }
            if d.stage != Stage::Basic as u8 {
                for t in &own {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            }
        } else if d.is_trainer() {
            if d.trainer_type == TrainerType::Tool as u8 {
                for t in &own {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            } else {
                out.push(Action::PlayCard { hand_index, target: BOARD });
            }
        }
    }

    let mut names: Vec<&'static str> = Vec::new();
    let mut from_names: Vec<(&'static str, &'static str)> = Vec::new();
    let add = |n: &'static str, names: &mut Vec<&'static str>| {
        if !names.contains(&n) {
            names.push(n);
        }
    };
    if let Some(c) = g.st.active_pokemon(p) {
        for a in g.st.cdef(c).attacks {
            add(a.name, &mut names);
        }
    }
    for &b in pl.bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            for a in g.st.cdef(c).attacks.iter().filter(|a| a.use_on_bench) {
                add(a.name, &mut names);
            }
        }
    }
    if g.kinds_present.has(crate::effects::k::CHECK_POKEMON_ATTACKS) || g.st.slot(p, pl.active).tools.len() > 0 {
        if let Ok(((attacks, copied), _)) = ctx.attack_list() {
            for a in attacks.iter() {
                let n = g.st.cdef(a.card).attacks[a.idx()].name;
                if copied.iter().any(|c| c == a) {
                    // Copied from a Benched Pokemon (Memory Helix): named by the source card too.
                    let from = g.st.cdef(a.card).full_name;
                    if !from_names.iter().any(|(x, f)| *x == n && *f == from) {
                        from_names.push((n, from));
                    }
                } else {
                    add(n, &mut names);
                }
            }
        }
    }
    // Same order as the oracle's `[...names, ...name + '\0' + from].sort()`.
    let mut keyed: Vec<(String, Action)> = Vec::new();
    for n in names {
        keyed.push((n.to_string(), Action::Attack { name: n, from: None }));
    }
    for (n, f) in from_names {
        keyed.push((format!("{}\u{0}{}", n, f), Action::Attack { name: n, from: Some(f) }));
    }
    keyed.sort_by(|a, b| {
        let x: Vec<u16> = a.0.encode_utf16().collect();
        let y: Vec<u16> = b.0.encode_utf16().collect();
        x.cmp(&y)
    });
    for (_, a) in keyed {
        out.push(a);
    }

    for t in &own {
        let slot = crate::prompts::get_target(&g.st, p, *t).unwrap();
        if let Some(c) = g.st.slot_pokemon(slot.p as usize, slot.s) {
            let mut pn: Vec<&'static str> = Vec::new();
            for pw in g.st.cdef(c).powers {
                if !pn.contains(&pw.name) {
                    pn.push(pw.name);
                }
            }
            // (The checked power list only removes powers: the printed names are the candidates.)
            js_sort(&mut pn);
            for n in pn {
                out.push(Action::UseAbility { name: n, target: *t });
            }
        }
    }
    if g.st.stadium_card().is_some() {
        out.push(Action::UseStadium);
    }
    for (i, b) in pl.bench.iter().enumerate() {
        if !pl.slots[*b as usize].cards.is_empty() {
            out.push(Action::Retreat { bench_index: i as u8 });
        }
    }
    out.push(Action::Pass);
    out
}

/// Legal turn options (deduplicated, in candidate order). Descriptors are built for the legal ones only.
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut ctx = Ctx::new(g);
    let mut seen: Vec<Action> = Vec::new();
    let mut out = Vec::new();
    for action in candidates(&mut ctx) {
        // Two candidates with one descriptor are one option: a card is told apart by its id, so a repeat is
        // the same action.
        if seen.contains(&action) {
            continue;
        }
        seen.push(action);
        if legal_in(&mut ctx, action) {
            out.push(TurnOption { desc: describe_action(g, action), action });
        }
    }
    out
}

/// Legality of one turn action: from the declared checks, else a trial on a
/// copy of the game.
pub fn is_legal(g: &Game, a: Action) -> bool {
    legal_in(&mut Ctx::new(g), a)
}

/// Index into `legal_stats::KINDS`.
fn stat_kind(g: &Game, a: Action) -> usize {
    match a {
        Action::PlayCard { hand_index, .. } => {
            let p = g.st.active_player as usize;
            let Some(card) = g.st.players[p].hand.get(hand_index as usize) else { return 3 };
            let d = g.st.cdef(card);
            if d.is_energy() {
                0
            } else if d.is_pokemon() {
                if d.stage == Stage::Basic as u8 { 1 } else { 2 }
            } else {
                match d.trainer_type() {
                    TrainerType::Item => 3,
                    TrainerType::Supporter => 4,
                    TrainerType::Stadium => 5,
                    TrainerType::Tool => 6,
                }
            }
        }
        Action::Attack { .. } => 7,
        Action::UseAbility { .. } | Action::UseTrainerAbility { .. } => 8,
        Action::UseStadium => 9,
        Action::Retreat { .. } => 10,
        Action::Pass => 11,
    }
}

fn legal_in(ctx: &mut Ctx, a: Action) -> bool {
    let g = ctx.g;
    let stats = crate::legal_stats::enabled();
    let kind = if stats { stat_kind(g, a) } else { 0 };
    let answer = match legal_fast(ctx, a) {
        Some(legal) => {
            if stats {
                crate::legal_stats::record(kind, false, legal, None);
            }
            legal
        }
        // Not declared: play it on a copy.
        None => match trial(g, a, true) {
            Ok(()) => {
                if stats {
                    crate::legal_stats::record_fallback(kind, true, None, ctx.why);
                }
                true
            }
            Err(e) => {
                if stats {
                    crate::legal_stats::record_fallback(kind, false, Some(e.0), ctx.why);
                }
                false
            }
        },
    };
    if verify_legal() {
        let full = trial(g, a, false);
        assert_eq!(
            answer,
            full.is_ok(),
            "legality differs from the full trial: {:?} (declared answer {}, trial {:?}, fallback reason {:?}) {}",
            a,
            answer,
            full,
            ctx.why,
            describe_for_verify(g, a)
        );
    }
    answer
}

/// The cards an action involves, for the verification failure message.
fn describe_for_verify(g: &Game, a: Action) -> String {
    let p = g.st.active_player as usize;
    match a {
        Action::PlayCard { hand_index, target } => {
            let c = g.st.players[p].hand.get(hand_index as usize).map(|c| g.st.cdef(c).full_name).unwrap_or("?");
            format!("[play {} onto {:?}; turn {}]", c, target_json(target), g.st.turn)
        }
        Action::Attack { name, from } => {
            let c = g.st.active_pokemon(p).map(|c| g.st.cdef(c).full_name).unwrap_or("?");
            format!("[attack {} {:?} by {}; turn {}]", name, from, c, g.st.turn)
        }
        Action::UseAbility { name, target } => format!("[ability {} at {:?}; turn {}]", name, target_json(target), g.st.turn),
        _ => format!("[turn {}]", g.st.turn),
    }
}

/// `PTCG_VERIFY_LEGAL=1`: check every answer against the full trial.
fn verify_legal() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("PTCG_VERIFY_LEGAL").map_or(false, |v| v == "1"))
}

/// Play the action on a copy; `fast` skips work that can't change the answer.
fn trial(g: &Game, a: Action, fast: bool) -> crate::game::R {
    let mut trial = g.fork();
    trial.trial = fast;
    trial.rng = crate::rng::Rng::zero();
    trial.act_trial(a)?;
    // Resolve info prompts (an ability's animation wait) so checks that run
    // after them count toward legality; stop at chance prompts (except the
    // Confusion flip, see below) and decisions.
    // Chance inside the trial draws fixed outcomes (`Rng::zero`, the oracle's
    // `Chance.trial`): a coin callback that throws after its "Coin flip
    // animation" wait would otherwise make legality depend on a draw, and the
    // real stream must not leak into the legal set.
    for _ in 0..100 {
        if trial.st.phase == crate::types::GamePhase::Finished {
            break;
        }
        match trial.pending() {
            crate::game::Pending::Info(i) => {
                trial.resolve(i, crate::prompts::Res::True)?;
            }
            // The Confusion flip's heads branch runs the attack: resolve it as
            // heads so an attack that throws is not offered to a Confused attacker.
            crate::game::Pending::Chance(i)
                if trial.prompts.as_slice()[i].message == "FLIP_CONFUSION"
                    && matches!(trial.prompts.as_slice()[i].kind, crate::prompts::PromptKind::CoinFlip) =>
            {
                trial.resolve(i, crate::prompts::Res::Bool(true))?;
            }
            _ => break,
        }
    }
    Ok(())
}

/// Legal actions without descriptors (fast path for the select interface).
pub fn legal_actions(g: &Game) -> Vec<TurnOption> {
    let mut ctx = Ctx::new(g);
    let mut out: Vec<TurnOption> = Vec::new();
    for c in candidates(&mut ctx) {
        if out.iter().any(|o| o.action == c) {
            continue;
        }
        if legal_in(&mut ctx, c) {
            out.push(TurnOption { desc: Value::Null, action: c });
        }
    }
    out
}
