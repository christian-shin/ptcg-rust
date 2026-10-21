//! Turn-level options: structural candidates, each answered from the declared
//! checks (`legal.rs`); a trial on a copy of the game only for what is not
//! declared (a counted fallback). Options come in a fixed order.

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
    candidates(&mut Ctx::new(g), &mut out);
    out
}

/// The candidates, reading the attack list through the decision's context.
fn candidates(ctx: &mut Ctx, out: &mut Vec<Action>) {
    let g = ctx.g;
    out.clear();
    let p = g.st.active_player as usize;
    let own = slot_targets(&g.st, p, PlayerType::BottomPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]);
    let pl = &g.st.players[p];
    let first_empty = pl.bench.iter().position(|b| pl.slots[*b as usize].cards().is_empty());
    for (hi, c) in pl.hand().iter().enumerate() {
        let d = g.st.cdef(c);
        let hand_index = hi as u8;
        if d.is_energy() {
            for t in own.iter() {
                out.push(Action::PlayCard { hand_index, target: *t });
            }
        } else if d.is_pokemon() {
            if let Some(i) = first_empty {
                out.push(Action::PlayCard { hand_index, target: CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8) });
            }
            if d.stage != Stage::Basic as u8 {
                for t in own.iter() {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            }
        } else if d.is_trainer() {
            if d.trainer_type == TrainerType::Tool as u8 {
                for t in own.iter() {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            } else {
                out.push(Action::PlayCard { hand_index, target: BOARD });
            }
        }
    }

    // The Abilities of the Pokémon in play (the Active Spot, then the Bench), then the Stadium.
    for t in own.iter() {
        let slot = crate::prompts::get_target(&g.st, p, *t).unwrap();
        if let Some(c) = g.st.slot_pokemon(slot.p as usize, slot.s) {
            // (The checked power list only removes powers: the printed powers are the candidates.)
            for i in 0..g.st.cdef(c).powers.len() {
                out.push(Action::UseAbility { power: PowerRef { card: c, index: i as u8 }, target: *t });
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

/// The legal actions of the active player's turn into `out` (cleared first), in candidate order. The candidates
/// are distinct by construction (a hand card is told apart by its index, a target occurs once per card, an attack
/// is deduplicated by its identity in `candidates`, an Ability is one per power and spot), so there is no
/// duplicate scan; `PTCG_VERIFY_LEGAL=1` asserts it.
///
/// The card lists the scan put in the propagation order (`decl::OrderCache`) are kept in the game's order cache, for the
/// next decisions of the same layout and the action taken.
pub fn legal_actions_into(g: &mut Game, out: &mut Vec<Action>) {
    let mut ctx = Ctx::new(g);
    legal_actions_with(&mut ctx, out);
    let fresh = ctx.into_orders();
    g.order_cache.keep(g.st.layout_gen, &fresh);
}

/// [`legal_actions_into`] over the decision's facts `ctx` (a caller that reads more of the decision afterwards, the
/// table view: `host::decision_into`).
pub(crate) fn legal_actions_with(ctx: &mut Ctx, out: &mut Vec<Action>) {
    out.clear();
    let mut cands = LEGAL_SCRATCH.with(|s| std::mem::take(&mut *s.borrow_mut()));
    candidates(ctx, &mut cands);
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

/// Legal turn options (in candidate order). Descriptors are built for the legal ones only. For the tools and the checks that
/// read a `&Game`: the scan's order lists are not kept (`legal_actions_into` keeps them).
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut acts = Vec::new();
    legal_actions_with(&mut Ctx::new(g), &mut acts);
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

/// Legal actions without descriptors, read from a `&Game` (as [`legal_turn_options`]).
pub fn legal_actions(g: &Game) -> Vec<TurnOption> {
    let mut acts = Vec::new();
    legal_actions_with(&mut Ctx::new(g), &mut acts);
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
