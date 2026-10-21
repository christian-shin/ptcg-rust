//! Turn-level options: structural candidates, each answered from the declared
//! checks (`legal.rs`); a trial on a copy of the game only for what is not
//! declared (a counted fallback). The legal scan leaves out the candidates a
//! rule refuses before any other check (`Gates`). Options come in a fixed order.

use crate::effects::PowerRef;
use crate::game::{Action, Game};
use crate::state::AttackRef;
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

/// The printed name of an attack (the trace boundary and messages only: actions carry the attack itself).
pub fn attack_name(g: &Game, attack: AttackRef) -> &'static str {
    g.st.cdef(attack.card).attacks[attack.idx()].name
}

/// The printed name of an Ability (the trace boundary and messages only).
pub fn power_name(g: &Game, power: PowerRef) -> &'static str {
    g.st.cdef(power.card).powers[power.index as usize].name
}

/// The descriptor's `a` field of a turn action (what the policies tell actions apart by).
pub fn action_kind(a: &Action) -> &'static str {
    match a {
        Action::PlayCard { .. } => "play",
        Action::Attack { .. } => "attack",
        Action::UseAbility { .. } => "ability",
        Action::UseStadium => "stadium",
        Action::Retreat => "retreat",
        Action::Pass => "pass",
    }
}

/// Canonical descriptor of a turn action (as a trace records it).
pub fn describe_action(g: &Game, a: Action) -> Value {
    let p = g.st.active_player as usize;
    let card = match a {
        Action::PlayCard { hand_index, .. } => Some(g.st.players[p].hand().as_slice()[hand_index as usize]),
        _ => None,
    };
    describe_action_of(g, a, card)
}

/// [`describe_action`] with the played card given (for a `PlayCard`; it no longer has to be in the hand).
pub fn describe_action_of(g: &Game, a: Action, card: Option<CardId>) -> Value {
    match a {
        Action::PlayCard { target, .. } => {
            let c = card.expect("PlayCard needs its card");
            json!({ "a": "play", "card": g.card_ref(c), "target": target_json(target) })
        }
        // The trace names an attack copied from a Benched Pokémon by that Pokémon's card too.
        Action::Attack { attack, copied: false } => json!({ "a": "attack", "name": attack_name(g, attack) }),
        Action::Attack { attack, copied: true } => json!({ "a": "attack", "name": attack_name(g, attack), "from": g.st.cdef(attack.card).full_name }),
        Action::UseAbility { power, target } => json!({ "a": "ability", "name": power_name(g, power), "source": target_json(target) }),
        Action::UseStadium => json!({ "a": "stadium" }),
        Action::Retreat => json!({ "a": "retreat" }),
        Action::Pass => json!({ "a": "pass" }),
    }
}

/// Structural candidates for the active player's turn, in the rules order (the selection interface's turn order): the
/// hand cards by hand index (each one's targets: the Active Spot, then the Bench in order), the Abilities of the
/// Pokémon in play and the Stadium by position, the attacks in list order, the retreat, passing.
pub fn candidate_actions(g: &Game) -> Vec<Action> {
    let mut out = Vec::with_capacity(32);
    candidates(&mut Ctx::new(g), &mut out, false);
    out
}

/// The candidates a rule refuses before any other check (speed plan L7, "class gates"), read from the check function the
/// action's own checks (legality and execution) call: such a candidate is illegal whatever else holds, so the legal scan
/// doesn't list it. The classes closed for the whole decision are read once (the fields); the others per candidate in
/// [`candidates`] (an Evolution card onto a spot, a Tool onto a full spot, a passive Ability). `PTCG_VERIFY_LEGAL=1` plays
/// every left-out candidate on a copy: it must be refused ([`left_out`]).
struct Gates {
    /// The gates are read (the legal scan); off: every structural candidate ([`candidate_actions`]).
    on: bool,
    /// The player attached an Energy from the hand this turn (`turn::energy_attached_this_turn`, read by
    /// `turn::can_attach_energy`): no Energy card can be attached.
    energy: bool,
    /// A Supporter / a Stadium was played this turn (`play_trainer::played_kind_this_turn`, read by
    /// `play_trainer::turn_rules`): no other card of the type can be played.
    supporter: bool,
    stadium: bool,
    /// No card in the game can lift "evolves from" (`enter::may_permit` of `Limit::EvolvesFrom`, read by
    /// `enter::evolve_rules`): an Evolution card is played only onto a Pokémon it evolves from (`enter::evolves_into`).
    evolves_from: bool,
}

impl Gates {
    fn of(g: &Game, p: usize) -> Gates {
        Gates {
            on: true,
            energy: crate::engine::turn::energy_attached_this_turn(g, p),
            supporter: crate::engine::play_trainer::played_kind_this_turn(g, p, TrainerType::Supporter),
            stadium: crate::engine::play_trainer::played_kind_this_turn(g, p, TrainerType::Stadium),
            evolves_from: !crate::engine::enter::may_permit(g, crate::spec::event::Limit::EvolvesFrom),
        }
    }

    /// Nothing is left out (the structural candidates, [`candidate_actions`]).
    const OFF: Gates = Gates { on: false, energy: false, supporter: false, stadium: false, evolves_from: false };
}

/// `PTCG_VERIFY_LEGAL=1`: a candidate the gates leave out is refused by the full trial.
fn left_out(g: &Game, a: Action) {
    if verify_legal() {
        let full = trial(g, a, false);
        assert!(full.is_err(), "a class gate left out a legal candidate: {:?} {}", a, describe_for_verify(g, a));
    }
}

/// The candidates, reading the attack list through the decision's context; `gated`: without those a rule refuses before
/// any other check ([`Gates`]; the legal scan).
fn candidates(ctx: &mut Ctx, out: &mut Vec<Action>, gated: bool) {
    let g = ctx.g;
    out.clear();
    let p = g.st.active_player as usize;
    let own = slot_targets(&g.st, p, PlayerType::BottomPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]);
    let pl = &g.st.players[p];
    let gates = if gated { Gates::of(g, p) } else { Gates::OFF };
    let first_empty = pl.bench.iter().position(|b| pl.slots[*b as usize].cards().is_empty());
    for (hi, c) in pl.hand().iter().enumerate() {
        let d = g.st.cdef(c);
        let hand_index = hi as u8;
        if d.is_energy() {
            for t in own.iter() {
                let a = Action::PlayCard { hand_index, target: *t };
                if gates.energy {
                    left_out(g, a);
                    continue;
                }
                out.push(a);
            }
        } else if d.is_pokemon() {
            // A card that isn't a Basic Pokémon is played onto a Pokémon only, by evolving it (`enter::hand_play_view`): the
            // gated scan leaves out its empty Bench spot, and, when no card lifts "evolves from", every spot holding a
            // Pokémon it doesn't evolve from (`enter::evolves_into`, read by `enter::evolve_rules`).
            let evolution = d.stage != Stage::Basic as u8;
            if let Some(i) = first_empty {
                let a = Action::PlayCard { hand_index, target: CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8) };
                if gates.on && evolution {
                    left_out(g, a);
                } else {
                    out.push(a);
                }
            }
            if evolution {
                for t in own.iter() {
                    let a = Action::PlayCard { hand_index, target: *t };
                    if gates.evolves_from {
                        let base = crate::prompts::get_target(&g.st, p, *t).ok().and_then(|s| g.st.slot_pokemon(s.p as usize, s.s));
                        if !base.is_some_and(|b| crate::engine::enter::evolves_into(g, b, c, crate::engine::enter::Reach::Next)) {
                            left_out(g, a);
                            continue;
                        }
                    }
                    out.push(a);
                }
            }
        } else if d.is_trainer() {
            if d.trainer_type == TrainerType::Tool as u8 {
                for t in own.iter() {
                    let a = Action::PlayCard { hand_index, target: *t };
                    // A Pokémon holding as many Tools as it may have takes no other (`attach::attach_spot_ok`, read by the
                    // Tool's Attach, `attach::check_attach_with`).
                    if gates.on {
                        let full = crate::prompts::get_target(&g.st, p, *t).map_or(true, |s| crate::engine::attach::attach_spot_ok(g, c, s, g.st.owner(c) as u8).is_err());
                        if full {
                            left_out(g, a);
                            continue;
                        }
                    }
                    out.push(a);
                }
            } else {
                let a = Action::PlayCard { hand_index, target: BOARD };
                let closed = match d.trainer_type() {
                    TrainerType::Supporter => gates.supporter,
                    TrainerType::Stadium => gates.stadium,
                    _ => false,
                };
                if closed {
                    left_out(g, a);
                    continue;
                }
                out.push(a);
            }
        }
    }

    // The Abilities of the Pokémon in play (the Active Spot, then the Bench), then the Stadium.
    for t in own.iter() {
        let slot = crate::prompts::get_target(&g.st, p, *t).unwrap();
        if let Some(c) = g.st.slot_pokemon(slot.p as usize, slot.s) {
            // (The checked power list only removes powers: the printed powers are the candidates.)
            for i in 0..g.st.cdef(c).powers.len() {
                let power = PowerRef { card: c, index: i as u8 };
                let a = Action::UseAbility { power, target: *t };
                // An Ability with nothing to use (a passive one) is never used (`turn::can_use_ability_core`).
                if gates.on && crate::engine::turn::can_use_ability_core(g, power, t.slot).is_err() {
                    left_out(g, a);
                    continue;
                }
                out.push(a);
            }
        }
    }
    if g.st.stadium_card().is_some() {
        out.push(Action::UseStadium);
    }
    // The attacks: the Active Pokémon's printed ones by index, then the derived ones (a Tool's, an earlier Stage's, a
    // Benched Pokémon's copied) in the order of the list. An attack a Pokémon card prints is one attack however many
    // cards of it are in play (two Benched copies of one card give one copied attack).
    let mut attacks: SVec<(AttackRef, bool), 64> = SVec::new();
    if g.decls.sets[crate::decl::fam::GRANT_ATTACKS] != 0 || g.st.slot(p, pl.active).tools().len() > 0 {
        if let Ok((_, all)) = ctx.attack_list() {
            for r in all.iter() {
                attacks.push(*r);
            }
        }
    }
    if attacks.is_empty() {
        if let Some(c) = g.st.active_pokemon(p) {
            for i in 0..g.st.cdef(c).attacks.len() {
                attacks.push((AttackRef { card: c, index: i as u8 }, false));
            }
        }
    }
    let def_of = |a: AttackRef| g.st.cards[a.card as usize].def;
    for i in 0..attacks.len() {
        let (attack, copied) = attacks.as_slice()[i];
        let seen = attacks.as_slice()[..i].iter().any(|(b, c)| *c == copied && b.index == attack.index && def_of(*b) == def_of(attack));
        if !seen {
            out.push(Action::Attack { attack, copied });
        }
    }
    // The retreat (one action: the Benched Pokémon is chosen after the cost is paid, APR A-03).
    if pl.bench.iter().any(|b| !pl.slots[*b as usize].cards().is_empty()) {
        out.push(Action::Retreat);
    }
    out.push(Action::Pass);
}

thread_local! {
    /// Scratch candidate list of `legal_actions_into`, kept between decisions (no allocation per decision).
    static LEGAL_SCRATCH: std::cell::RefCell<Vec<Action>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The legal actions of the active player's turn into `out` (cleared first), in candidate order (the candidates the
/// class gates leave, [`Gates`]). The candidates are distinct by construction (a hand card is told apart by its index, a target occurs once per card, an attack
/// is deduplicated by its identity in `candidates`, an Ability is one per power and spot), so there is no
/// duplicate scan; `PTCG_VERIFY_LEGAL=1` asserts it.
pub fn legal_actions_into(g: &Game, out: &mut Vec<Action>) {
    legal_actions_with(&mut Ctx::new(g), out);
}

/// [`legal_actions_into`] over the decision's facts `ctx` (a caller that reads more of the decision afterwards, the
/// table view: `host::decision_into`).
pub(crate) fn legal_actions_with(ctx: &mut Ctx, out: &mut Vec<Action>) {
    out.clear();
    let mut cands = LEGAL_SCRATCH.with(|s| std::mem::take(&mut *s.borrow_mut()));
    candidates(ctx, &mut cands, true);
    if verify_legal() {
        for (i, a) in cands.iter().enumerate() {
            assert!(!cands[..i].contains(a), "duplicate candidate: {:?}", a);
        }
    }
    for &action in cands.iter() {
        if legal_in(ctx, action) {
            out.push(action);
        }
    }
    LEGAL_SCRATCH.with(|s| *s.borrow_mut() = cands);
}

/// Legal turn options (in candidate order). Descriptors are built for the legal ones only.
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut acts = Vec::new();
    legal_actions_into(g, &mut acts);
    acts.into_iter().map(|action| TurnOption { desc: describe_action(g, action), action }).collect()
}

/// Index into `legal_stats::KINDS`.
fn stat_kind(g: &Game, a: Action) -> usize {
    match a {
        Action::PlayCard { hand_index, .. } => {
            let p = g.st.active_player as usize;
            let Some(card) = g.st.players[p].hand().get(hand_index as usize) else { return 3 };
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
        Action::UseAbility { .. } => 8,
        Action::UseStadium => 9,
        Action::Retreat => 10,
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
            let c = g.st.players[p].hand().get(hand_index as usize).map(|c| g.st.cdef(c).full_name).unwrap_or("?");
            format!("[play {} onto {:?}; turn {}]", c, target_json(target), g.st.turn)
        }
        Action::Attack { attack, copied } => {
            let c = g.st.active_pokemon(p).map(|c| g.st.cdef(c).full_name).unwrap_or("?");
            format!("[attack {} copied {} by {}; turn {}]", attack_name(g, attack), copied, c, g.st.turn)
        }
        Action::UseAbility { power, target } => format!("[ability {} at {:?}; turn {}]", power_name(g, power), target_json(target), g.st.turn),
        _ => format!("[turn {}]", g.st.turn),
    }
}

/// `PTCG_VERIFY_LEGAL=1`: check every answer against the full trial.
pub(crate) fn verify_legal() -> bool {
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
    // Chance inside the trial draws fixed outcomes (`Rng::zero`): a coin callback that throws after its "Coin flip
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
                if trial.prompts.as_slice()[i].purpose == crate::prompts::PromptPurpose::ConfusionFlip
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
    let mut acts = Vec::new();
    legal_actions_into(g, &mut acts);
    acts.into_iter().map(|action| TurnOption { desc: Value::Null, action }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::carddb::{cards, def, def_by_full_name, DefId};

    /// The rank of a turn action in the selection interface's turn order: the hand cards by hand index and target (the
    /// Active Spot first), then the Abilities and the Stadium by position (the Active Spot, the Bench, the Stadium),
    /// then the attacks, then the retreat, then passing.
    fn rank(a: Action) -> (u8, u8, u8) {
        let slot = |t: CardTarget| match t.slot {
            SlotType::Active => 0,
            SlotType::Bench => 1 + t.index,
            _ => 100,
        };
        match a {
            Action::PlayCard { hand_index, target } => (0, hand_index, slot(target)),
            Action::UseAbility { power, target } => (1, slot(target), power.index),
            Action::UseStadium => (1, 200, 0),
            Action::Attack { attack, .. } => (2, attack.idx() as u8, 0),
            Action::Retreat => (3, 0, 0),
            Action::Pass => (4, 0, 0),
        }
    }

    /// The candidates and the legal options come in the selection interface's turn order on a constructed board, with the attacks in
    /// printed order although their names sort the other way.
    #[test]
    fn candidates_come_in_the_turn_action_order() {
        // A Pokémon whose second attack sorts before its first by name, a Stadium, and Pokémon with an Ability.
        let attacker = cards()
            .iter()
            .position(|c| c.is_pokemon() && c.stage == Stage::Basic as u8 && c.attacks.len() >= 2 && c.attacks[0].name > c.attacks[1].name)
            .expect("a Pokémon whose attacks are not printed in name order") as DefId;
        let stadium = cards().iter().position(|c| c.is_trainer() && c.trainer_type() == TrainerType::Stadium).expect("a Stadium") as DefId;
        let name = |d: DefId| def(d).full_name;
        let id = |n: &str| def_by_full_name(n).unwrap();
        let mut deck: Vec<DefId> = vec![attacker, stadium, id("Metang TEF 114"), id("Metang TEF 114"), id("Beldum TEF 113"), id("Beldum TEF 113")];
        deck.resize(60, id("Metal Energy MEE 8"));
        let mut checked = 0;
        for seed in 1..40 {
            let mut g = Game::new(seed);
            g.start([&deck, &deck]).unwrap();
            // Answer the setup prompts (Active and Bench choices) until the first turn decision: a trial of an
            // action refuses to run while a prompt is open.
            let mut stuck = false;
            while let Some(sel) = g.select_with(false).unwrap() {
                if sel.select_type == crate::interface::SelectType::Main {
                    break;
                }
                let mut picks = Vec::new();
                while picks.len() < sel.min_count {
                    let (mask, _) = g.pick_mask(&sel, &picks);
                    picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
                }
                if g.answer(&sel, &picks).is_err() {
                    stuck = true;
                    break;
                }
            }
            if stuck || !matches!(g.pending(), crate::game::Pending::Turn(0)) {
                continue;
            }
            let sc = json!({
                "me": {"reset": true, "active": name(attacker), "active_energy": ["Metal Energy MEE 8", "Metal Energy MEE 8"],
                       "bench": [{"card": "Metang TEF 114"}, {"card": "Beldum TEF 113"}, {"card": "Metang TEF 114"}],
                       "hand": ["Metal Energy MEE 8", "Beldum TEF 113", "Metal Energy MEE 8"], "stadium": name(stadium)},
                "opp": {"reset": true, "active": "Beldum TEF 113"}
            });
            if crate::scenario::apply(&mut g, &sc).is_err() {
                continue;
            }
            let cands = candidate_actions(&g);
            let ranks: Vec<_> = cands.iter().map(|a| rank(*a)).collect();
            assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "candidates out of order: {:?}", ranks);
            let groups: Vec<u8> = ranks.iter().map(|r| r.0).collect();
            for want in 0..5u8 {
                assert!(groups.contains(&want), "the board offers no candidate of group {want}");
            }
            let attacks: Vec<usize> = cands.iter().filter_map(|a| if let Action::Attack { attack, .. } = a { Some(attack.idx()) } else { None }).collect();
            let printed: Vec<usize> = (0..def(attacker).attacks.len()).collect();
            assert_eq!(attacks, printed, "the attacks come in printed order");
            let legal: Vec<_> = legal_actions(&g).iter().map(|o| rank(o.action)).collect();
            assert!(legal.windows(2).all(|w| w[0] <= w[1]), "legal options out of order: {:?}", legal);
            assert_eq!(legal.last(), Some(&(4, 0, 0)));
            checked += 1;
        }
        assert!(checked > 0, "no board was built");
    }

    /// The class gates leave out illegal candidates only: at every turn decision of random games between the benchmark's
    /// decks (bench/spec.json), the legal scan's list equals every structural candidate answered one by one, and the gates
    /// leave some out.
    #[test]
    fn class_gates_leave_out_illegal_candidates_only() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../bench/spec.json")).unwrap();
        let decks: Vec<Vec<DefId>> = spec["decks"].as_array().unwrap().iter().map(|d| crate::selfplay::expand_deck(&d["cards"]).unwrap()).collect();
        let n = decks.len();
        let (mut decisions, mut left_out) = (0usize, 0usize);
        for seed in 0..n as u32 * 2 {
            let mut g = Game::new(seed);
            g.start([&decks[seed as usize % n], &decks[(seed as usize / 2 + 1) % n]]).unwrap();
            g.settle().ok();
            let mut rng = crate::rng::Rng::new(seed.wrapping_mul(2654435761));
            let mut gated = Vec::new();
            for _ in 0..3000 {
                if let crate::game::Pending::Turn(_) = g.pending() {
                    legal_actions_into(&g, &mut gated);
                    let mut ctx = Ctx::new(&g);
                    let mut all = Vec::new();
                    candidates(&mut ctx, &mut all, false);
                    let mut gated_cands = Vec::new();
                    candidates(&mut ctx, &mut gated_cands, true);
                    left_out += all.len() - gated_cands.len();
                    let open: Vec<Action> = all.iter().copied().filter(|a| legal_in(&mut ctx, *a)).collect();
                    assert_eq!(gated, open, "seed {seed}, turn {}", g.st.turn);
                    decisions += 1;
                    let a = gated[rng.index(gated.len())];
                    if g.act(a).and_then(|_| g.settle()).is_err() {
                        break;
                    }
                    continue;
                }
                let Ok(Some(sel)) = g.select() else { break };
                let mut picks: Vec<usize> = Vec::new();
                while picks.len() < sel.min_count {
                    let (mask, _) = g.pick_mask(&sel, &picks);
                    let allowed: Vec<usize> = (0..sel.options.len()).filter(|j| mask[*j]).collect();
                    picks.push(allowed[rng.index(allowed.len())]);
                }
                if g.answer(&sel, &picks).and_then(|_| g.settle()).is_err() {
                    break;
                }
            }
        }
        assert!(decisions > 500, "{decisions} turn decisions");
        assert!(left_out > decisions, "the gates left out {left_out} candidates in {decisions} turn decisions");
    }

    /// The candidates of every turn decision of 12 games are pairwise distinct (why `legal_actions_into` has no
    /// duplicate scan), and the evaluations per decision are the candidates: the counts the cost is read from
    /// (`cargo test ... candidate_evaluations -- --nocapture`).
    #[test]
    fn candidate_evaluations_are_distinct_candidates() {
        use crate::interface::SelectType;
        let names: Vec<&str> = (0..60)
            .map(|i| match i {
                0..=3 => "Duraludon PRE 69",
                4..=6 => "Team Rocket's Mewtwo ex DRI 81",
                7..=10 => "Sacred Charm PFL 93",
                _ => "Metal Energy MEE 8",
            })
            .collect();
        let deck: Vec<u16> = names.iter().map(|n| def_by_full_name(n).unwrap()).collect();
        let (mut decisions, mut candidates, mut tail_compares) = (0usize, 0usize, 0usize);
        for seed in 0..12u32 {
            let mut g = Game::new(seed);
            g.start([&deck, &deck]).unwrap();
            let mut x = (seed as u64).wrapping_mul(0x9E3779B97F4A7C15) | 1;
            while let Some(sel) = g.select_with(false).unwrap() {
                let mut picks = Vec::new();
                if sel.select_type == SelectType::Main {
                    let cands = candidate_actions(&g);
                    for (i, a) in cands.iter().enumerate() {
                        assert!(!cands[..i].contains(a), "duplicate candidate {:?}", a);
                        // What the retired duplicate scan compared: the tail run of the earlier candidates.
                        tail_compares += match a {
                            Action::PlayCard { hand_index, .. } => cands[..i]
                                .iter()
                                .rev()
                                .take_while(|y| matches!(y, Action::PlayCard { hand_index: h, .. } if h == hand_index))
                                .count(),
                            _ => cands[..i].iter().rev().take_while(|y| !matches!(y, Action::PlayCard { .. })).count(),
                        };
                    }
                    decisions += 1;
                    candidates += cands.len();
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    picks.push((x % sel.options.len() as u64) as usize);
                } else {
                    while picks.len() < sel.min_count {
                        let (mask, _) = g.pick_mask(&sel, &picks);
                        picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
                    }
                }
                g.answer(&sel, &picks).unwrap();
            }
        }
        println!("turn decisions {decisions}, candidates evaluated {candidates}, retired duplicate-scan comparisons {tail_compares}");
        assert!(decisions > 0);
    }
}
