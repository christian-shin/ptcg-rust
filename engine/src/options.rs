//! Turn-level options: structural candidates filtered by trial dispatch on a
//! clone (cheap: the whole game is `Copy`). Mirrors the oracle's
//! `turnCandidates` + `legalTurnOptions` exactly, including order.

use crate::effects::*;
use crate::engine::turn::check_attacks_effect;
use crate::game::{Action, Game};
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
        let mut sim = g.fork();
        if let Ok((Effect::CheckPokemonAttacks { attacks, copied, .. }, _)) = { let e = check_attacks_effect(&sim, p); sim.run_fx(e) } {
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
            if g.kinds_present.has(crate::effects::k::CHECK_POKEMON_POWERS) {
                let mut sim = g.fork();
                let mut powers = SVec::new();
                for i in 0..g.st.cdef(c).powers.len() {
                    powers.push(PowerRef { card: c, index: i as u8 });
                }
                if let Ok((Effect::CheckPokemonPowers { powers, .. }, _)) = sim.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers }) {
                    for r in powers.iter() {
                        let n = g.st.cdef(r.card).powers[r.index as usize].name;
                        if !pn.contains(&n) {
                            pn.push(n);
                        }
                    }
                }
            }
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

/// Legal turn options (deduplicated, in candidate order).
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut ctx = Pass::default();
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for c in turn_candidates(g) {
        let key = serde_json::to_string(&c.desc).unwrap();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        if legal_in(g, c.action, &mut ctx) {
            out.push(c);
        }
    }
    out
}

/// Legality of one turn action: fast answers where they are certain,
/// otherwise a trial on a copy of the game.
pub fn is_legal(g: &Game, a: Action) -> bool {
    legal_in(g, a, &mut Pass::default())
}

/// What one pass over a decision's candidates learns for the next ones.
#[derive(Default)]
struct Pass {
    /// A retreat failed paying its cost, which doesn't depend on the Benched
    /// Pokémon chosen: every other retreat fails the same way.
    retreat_cost_unpaid: bool,
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

fn legal_in(g: &Game, a: Action, ctx: &mut Pass) -> bool {
    let stats = crate::legal_stats::enabled();
    let kind = if stats { stat_kind(g, a) } else { 0 };
    let fast = match a {
        // Ending the turn is always possible.
        Action::Pass => {
            if stats {
                crate::legal_stats::record(kind, false, true, None);
            }
            true
        }
        Action::Retreat { .. } if ctx.retreat_cost_unpaid => {
            if stats {
                crate::legal_stats::record(kind, false, false, None);
            }
            false
        }
        _ if rejects(g, a) => {
            if stats {
                crate::legal_stats::record(kind, false, false, None);
            }
            false
        }
        _ => match trial(g, a, true) {
            Ok(()) => {
                if stats {
                    crate::legal_stats::record(kind, true, true, None);
                }
                true
            }
            Err(e) => {
                if matches!(a, Action::Retreat { .. }) && e.0 == "NOT_ENOUGH_ENERGY" {
                    ctx.retreat_cost_unpaid = true;
                }
                if stats {
                    crate::legal_stats::record(kind, true, false, Some(e.0));
                }
                false
            }
        },
    };
    if verify_legal() {
        assert_eq!(fast, trial(g, a, false).is_ok(), "fast legality differs from the full trial: {:?}", a);
    }
    fast
}

/// `PTCG_VERIFY_LEGAL=1`: check every fast answer against the full trial.
fn verify_legal() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("PTCG_VERIFY_LEGAL").map_or(false, |v| v == "1"))
}

/// Actions the trial would certainly reject, decided without playing them.
/// Each check mirrors an unconditional failure of the trial's code path;
/// `PTCG_VERIFY_LEGAL=1` checks them against the trial.
fn rejects(g: &Game, a: Action) -> bool {
    use crate::engine::{play, retreat, turn};
    let p = g.st.active_player as usize;
    match a {
        Action::PlayCard { hand_index, target } => {
            let Some(card) = g.st.players[p].hand.get(hand_index as usize) else { return false };
            let d = g.st.cdef(card);
            if d.is_energy() {
                return turn::can_attach_energy(g, p, target).is_err();
            }
            if d.is_pokemon() {
                let Ok(t) = crate::prompts::get_target(&g.st, p, target) else { return true };
                return play::can_play_pokemon(g, p, card, t).is_err();
            }
            // The turn rules checked before the Trainer's effect is dispatched.
            match d.trainer_type() {
                TrainerType::Supporter => turn::can_play_supporter_card(g, p, card).is_err(),
                TrainerType::Stadium => turn::can_play_stadium_card(g, p, card).is_err(),
                _ => false,
            }
        }
        Action::UseStadium => turn::can_use_stadium(g, p).is_err(),
        // retreat::reducer: its state checks (card handlers only add blocks).
        Action::Retreat { bench_index } => retreat::can_retreat(g, p, bench_index, false).is_err(),
        _ => false,
    }
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
    let mut ctx = Pass::default();
    let mut out: Vec<TurnOption> = Vec::new();
    for c in turn_candidates_fast(g) {
        if out.iter().any(|o| o.action == c) {
            continue;
        }
        if legal_in(g, c, &mut ctx) {
            out.push(TurnOption { desc: Value::Null, action: c });
        }
    }
    out
}

fn turn_candidates_fast(g: &Game) -> Vec<Action> {
    candidate_actions(g)
}
