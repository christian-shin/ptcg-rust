//! Turn structure: draw, end of turn, between
//! turns, special conditions.
//!
//! Fields the end of a turn would clear that no pool card ever sets are not modeled; they are listed next to the code
//! that would touch them.

use crate::effects::*;
use crate::game::{Cont, Game, OnComplete, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

pub fn end_game(g: &mut Game, winner: Winner) {
    if g.st.phase == GamePhase::Finished {
        return;
    }
    crate::record::game_end(g, winner, g.st.end_reason);
    g.st.winner = winner;
    g.st.phase = GamePhase::Finished;
    // The game flow's writes are writes too (the derived facts are stale: `State::version`).
    g.st.bump();
    crate::expect::on_game_end(g);
}

pub fn init_next_turn(g: &mut Game) -> R {
    if g.st.phase != GamePhase::Setup && g.st.phase != GamePhase::BetweenTurns {
        return Ok(());
    }
    if g.st.phase == GamePhase::BetweenTurns {
        // usedTurnSkip is not modeled (no pool card sets it).
        g.st.active_player ^= 1;
    }
    let p = g.st.active_player as usize;
    g.st.turn += 1;
    g.st.players[p].moved_to_active_this_turn.clear();
    g.st.players[p].moved_from_active_to_bench_this_turn.clear();
    g.st.bump();

    g.st.phase = GamePhase::Draw;
    g.st.bump();
    if g.st.players[p].deck.is_empty() {
        let winner = if g.st.active_player != 0 { WINNER_P1 } else { WINNER_P2 };
        crate::engine::knockout::game_end(g, winner, crate::engine::knockout::EndReason::DeckOut);
        return Ok(());
    }
    // The BeginTurn event (batch 8: no card reacts to it in the pool).
    g.run_fx_unit(Effect::BeginTurn { p: p as u8 })?;

    // The turn's draw (APR H, G step 4): a Draw event by the rule.
    crate::engine::cards_zone::draw(g, p, 1, crate::cause::Cause::rule(crate::cause::RuleWhich::TurnDraw, p as u8))?;
    g.wait(p, Cont::PhasePlayerTurn);
    Ok(())
}

fn start_next_turn(g: &mut Game) -> R {
    let p = g.st.active_player as usize;
    let a = g.st.players[p].active;
    // Pokémon Checkup: a Pokémon Paralyzed since the start of its owner's last turn recovers.
    crate::engine::condition::remove(g, SlotRef::new(p, a), SpecialCondition::Paralyzed, crate::engine::condition::by_checkup(p))?;
    // A card left in the play area goes to the discard pile with the end of the turn.
    crate::record::scope(g, crate::spec::event::EventKind::EndTurn, p as u8, None, |g| g.move_to(ListRef::Supporter(p as u8), ListRef::Discard(p as u8), None));
    between_turns(g, OnComplete::InitNextTurn)
}

pub fn between_turns(g: &mut Game, oc: OnComplete) -> R {
    let entered = g.st.phase == GamePhase::PlayerTurn || g.st.phase == GamePhase::Attack;
    if entered {
        g.st.phase = GamePhase::BetweenTurns;
        g.st.bump();
        g.wait(g.st.active_player as usize, Cont::BetweenTurnsWait { oc });
        return Ok(());
    }
    run_between_turns_effects(g, oc)
}

pub fn run_between_turns_effects(g: &mut Game, oc: OnComplete) -> R {
    // APR F step 2 (E-08, id2302): two or more Checkup effects that apply are ordered by the player whose turn would be
    // next, each before or after the Special Conditions check (batch 8 C11c); otherwise the order of today.
    // Asked only when several effects apply (APR F step 2: "If there are several such effects to apply, the player whose
    // turn would be next can decide in which order to apply them, as well as decide for each of them whether they should
    // be applied before or after checking all Special Conditions"; user decision 7): two or more distinct effects
    // (identical ones are one: the order between them can't change the game; batch 8 C12b, review 8b Must 4). The
    // Special Conditions check is then an item only when an Active Pokémon has one (`order_prompt`).
    let mut held = false;
    for p in 0..2 {
        let a = g.st.players[p].active;
        let slot = g.st.slot(p, a);
        let id = g.new_fx(Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage });
        let t = crate::spec::run::timed_triggers(g, id, crate::effects::k::BETWEEN_TURNS);
        g.release_fx(id);
        held |= t?.iter().any(|(_, applies)| *applies);
    }
    // Any Checkup effect that applies goes through the ordered path: with one effect (or identical ones and no Special
    // Condition) nothing is asked and the rulebook's order holds, the Special Conditions first (APR F step 1), then the
    // effects (step 2; user decision 2026-10-13, batch 8 C12d).
    if held {
        return ordered_checkup(g, oc);
    }
    for p in 0..2 {
        let a = g.st.players[p].active;
        let slot = g.st.slot(p, a);
        // The Checkup event of player `p` (batch 8: its triggers run after it), carrying the Poison and Burn amounts the
        // "N more damage counters" declarations raise (`checkup_damage`, read before it); then the Special Conditions.
        let id = g.new_fx(Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage });
        let r = checkup_damage(g, id).and_then(|_| g.reduce_effect(id));
        let (poison, burn) = match *g.e(id) {
            Effect::BetweenTurns { poison_damage, burn_damage, .. } => (poison_damage, burn_damage),
            _ => (10, 20),
        };
        g.release_fx(id);
        r?;
        handle_special_conditions(g, p, poison, burn)?;
    }
    if g.has_prompts() {
        g.wait_prompt(Cont::BetweenTurnsCheck { oc });
        return Ok(());
    }
    crate::engine::knockout::state_check(g, oc)
}

/// Pokémon Checkup with two or more effects to order: each player's Checkup event (its amounts, the triggers that aren't
/// effects), then the order prompt over the effects and the Special Conditions check.
fn ordered_checkup(g: &mut Game, oc: OnComplete) -> R {
    let mut poison = [10, 10];
    let mut burn = [20, 20];
    let mut ids = [0 as EffId; 2];
    let mut items: SVec<(crate::engine::order::Item, bool), 32> = SVec::new();
    g.checkup_held = true;
    let r = (|| -> R {
        for p in 0..2 {
            let a = g.st.players[p].active;
            let slot = g.st.slot(p, a);
            let id = g.new_fx(Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage });
            ids[p] = id;
            checkup_damage(g, id)?;
            for (t, x) in crate::spec::run::timed_triggers(g, id, crate::effects::k::BETWEEN_TURNS)?.iter().copied() {
                if x {
                    items.push((crate::engine::order::Item::Card(t), true));
                }
            }
            g.reduce_effect(id)?;
            if let Effect::BetweenTurns { poison_damage, burn_damage, .. } = *g.e(id) {
                poison[p] = poison_damage;
                burn[p] = burn_damage;
            }
        }
        Ok(())
    })();
    g.checkup_held = false;
    r?;
    // Every effect that applies is in the window; the order is asked only when two or more are distinct (`several`).
    let several = crate::engine::order::asks(g, items.as_slice());
    let w = crate::engine::order::Window::Checkup { oc, poison, burn, e: ids, several, sc: true };
    crate::engine::order::collect(g, w.tag(), items.as_slice());
    crate::engine::order::next(g, w)
}

/// Is the Special Conditions check an item of the Checkup order (an Active Pokémon has a Special Condition)?
pub(crate) fn special_conditions_item(g: &Game) -> bool {
    (0..2).any(|p| !g.st.slot(p, g.st.players[p].active).special_conditions.is_empty())
}

/// `oc` for EndTurn's checkState, after KO resolution.
pub fn after_end_turn(g: &mut Game, p: usize) -> R {
    // Scenario `expect` assertions at "turn_end": Knock Outs are done, Pokémon Checkup is next.
    crate::expect::on_turn_end(g);
    // Expire the effects that last until a Knock Out of the opponent's Pokémon (denyPrizes: not modeled).
    crate::engine::lasting::after_end_turn(&mut g.st, p);
    g.st.bump();
    if g.st.phase == GamePhase::Finished {
        return Ok(());
    }
    start_next_turn(g)
}

/// The Active Pokémon of `p` recovers from `sc` at Pokémon Checkup (a heads for Burned or Asleep): the
/// RemoveCondition event.
pub fn checkup_recovers(g: &mut Game, p: usize, sc: SpecialCondition) -> R {
    let a = g.st.players[p].active;
    crate::engine::condition::remove(g, SlotRef::new(p, a), sc, crate::engine::condition::by_checkup(p))?;
    Ok(())
}

/// Scenario setup only (a board edit, not an event): the slot's Pokémon is affected by `sc` with its rule's
/// counters. Every game path gains a condition through `engine::condition::gain`.
pub fn add_condition(slot: &mut Slot, sc: SpecialCondition) {
    crate::engine::condition::put_condition(slot, sc, crate::engine::condition::base_counters(sc));
}

/// Would making the slot's Pokémon
/// `conds` change the game state? Re-applying a condition it has changes nothing, except an "enhanced" one
/// (adding Poisoned / Burned / Confused sets that condition's counters back to 1 / 2 / 3).
pub fn would_change_special_conditions(slot: &Slot, conds: &[SpecialCondition]) -> bool {
    conds.iter().any(|sc| {
        if !slot.special_conditions.contains(&(*sc as u8)) {
            return true;
        }
        (*sc == SpecialCondition::Poisoned && slot.poison_damage != 10)
            || (*sc == SpecialCondition::Burned && slot.burn_damage != 20)
            || (*sc == SpecialCondition::Confused && slot.confusion_damage != 30)
    })
}

/// The "put N more damage counters on each Poisoned / Burned Pokémon during Pokémon Checkup" declarations
/// (`Modifier::CheckupDamage`: Perilous Jungle, Pecharunt, Magmortar), in the propagation order of the declaring cards (any
/// zone: each checks where its card is), on the Checkup event `id` of a player before it is made.
fn checkup_damage(g: &mut Game, id: EffId) -> R {
    let order = g.decl_all(crate::decl::fam::CHECKUP_DAMAGE, 0);
    if order.is_empty() {
        return Ok(());
    }
    let before = *g.e(id);
    checkup_damage_from(g, id, &order)?;
    // VERIFY_LEGAL (plan 2.5): the amounts don't count an opponent's Ability a `Prevent` keeps off the Active Pokémon.
    if let Effect::BetweenTurns { p, poison_damage, burn_damage } = *g.e(id) {
        let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
        let skip = crate::derived::reach_skip_for(g, target);
        if skip != 0 {
            let mut h = g.fork();
            h.reach_skip = skip;
            *h.e_mut(id) = before;
            if checkup_damage_from(&mut h, id, &order).is_ok() {
                if let Effect::BetweenTurns { poison_damage: q, burn_damage: b, .. } = *h.e(id) {
                    assert_eq!((poison_damage, burn_damage), (q, b), "Checkup damage: an opponent's Ability a Prevent keeps off {target:?} added counters");
                }
            }
        }
    }
    Ok(())
}

fn checkup_damage_from(g: &mut Game, id: EffId, order: &SVec<CardId, 120>) -> R {
    use crate::spec::passive::Modifier;
    crate::derived::Facts::undelegated(g, |g| -> R {
        for c in order.iter().copied() {
            for ps in crate::decl::passives(g, c) {
                if let Modifier::CheckupDamage(_) = &ps.modifier {
                    crate::spec::passive::apply(g, c, id, ps)?;
                }
            }
        }
        Ok(())
    })
}

/// Pokémon Checkup's Special Conditions for player `p`'s Active Pokémon, with the Poison and Burn amounts of its Checkup
/// event.
fn handle_special_conditions(g: &mut Game, p: usize, poison: i32, burn: i32) -> R {
    // Iterate over a snapshot, like `for...of` over the original array.
    let a0 = g.st.players[p].active;
    let conds = g.st.slot(p, a0).special_conditions;
    // Pokémon Checkup step 1 (Advanced Player's Rulebook I-F): Poisoned, Burned, Asleep, Paralyzed,
    // whatever order the Special Conditions were applied in (the coin flips follow this order).
    let order = [SpecialCondition::Poisoned, SpecialCondition::Burned, SpecialCondition::Asleep, SpecialCondition::Paralyzed];
    for &sp in order.iter().filter(|c| conds.contains(&(**c as u8))).map(|c| *c as u8).collect::<Vec<u8>>().iter() {
        let a = g.st.players[p].active;
        // Poison's and Burn's damage counters: a PlaceCounters by the Special Condition (APR F; events batch 6).
        let by_condition = crate::engine::condition::by_condition(p);
        match SpecialCondition::from_u8(sp) {
            SpecialCondition::Poisoned => {
                crate::engine::damage::place(g, SlotRef::new(p, a), poison, by_condition)?;
            }
            SpecialCondition::Burned => {
                crate::engine::damage::place(g, SlotRef::new(p, a), burn, by_condition)?;
                g.prompt(p, PromptPurpose::BurnFlip, PromptKind::CoinFlip, crate::prompts::Provenance::NONE, Cont::BurnFlip { p: p as u8, slot: a });
            }
            SpecialCondition::Asleep => {
                let flips = g.st.slot(p, a).sleep_flips.max(0) as usize;
                if flips > 0 {
                    let prompts: Vec<(usize, PromptPurpose, PromptKind)> = (0..flips).map(|_| (p, PromptPurpose::SleepFlip, PromptKind::CoinFlip)).collect();
                    g.prompt_group(&prompts, Cont::SleepFlips { p: p as u8, slot: a });
                } else {
                    checkup_recovers(g, p, SpecialCondition::Asleep)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The end of player `p`'s turn: the EndTurn event (its consequences, then the triggers "at the end of your turn"),
/// then the end-of-turn processing (`end_turn`: the turn's records, the lasting effects' timing, the state check, the
/// next turn). Every way a turn ends calls it.
pub fn end_turn_event(g: &mut Game, p: usize) -> R {
    g.run_fx_unit(Effect::EndTurn { p: p as u8 })?;
    if std::mem::take(&mut g.order_pending) {
        // Two or more end-of-turn effects: the turn player orders them; the end of the turn continues after the last.
        return Ok(());
    }
    end_turn(g, p)
}

/// The Special Conditions check of Pokémon Checkup for both players, with their amounts: done once (APR F step 1), by the
/// window's first step or the order's choice.
pub(crate) fn checkup_conditions(g: &mut Game, w: &mut crate::engine::order::Window) -> R {
    if let crate::engine::order::Window::Checkup { poison, burn, sc, .. } = w {
        *sc = false;
        for p in 0..2 {
            handle_special_conditions(g, p, poison[p], burn[p])?;
        }
    }
    Ok(())
}

/// What follows a chained window whose effects are all applied: the rest of the end of the turn (the EndTurn event `e`
/// released), or the rest of Pokémon Checkup.
pub(crate) fn window_done(g: &mut Game, w: crate::engine::order::Window) -> R {
    match w {
        crate::engine::order::Window::EndTurn { p, e } => {
            g.release_fx(e);
            end_turn(g, p as usize)
        }
        crate::engine::order::Window::Checkup { oc, e, .. } => {
            g.release_fx(e[0]);
            g.release_fx(e[1]);
            checkup_finish(g, oc)
        }
        // The attack's and the state check's windows end in their callers' loops.
        crate::engine::order::Window::Damaged { .. } | crate::engine::order::Window::KnockOut => Ok(()),
    }
}

/// After Pokémon Checkup: the state check, once the prompts it opened are answered.
fn checkup_finish(g: &mut Game, oc: OnComplete) -> R {
    if g.has_prompts() {
        g.wait_prompt(Cont::BetweenTurnsCheck { oc });
        return Ok(());
    }
    crate::engine::knockout::state_check(g, oc)
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::EndTurn { p } => {
            // The turn's usage records end before the end-of-turn effects the player orders: an expiry is not an effect (APR
            // E-21).
            crate::engine::usage::expire_this_turn(&mut g.st, p as usize);
            crate::engine::lasting::expire_this_turn(&mut g.st, p as usize);
            Ok(())
        }
        Effect::AfterAttack { opp, .. } => {
            // A Pokémon that survived this attack on 10 HP keeps 10 HP when an effect of the
            // attack lowered its maximum HP (ruling 1589).
            let survivors = std::mem::replace(&mut g.ten_hp, SVec::new());
            for t in survivors.iter() {
                let (tp, ts) = (t.p as usize, t.s);
                if g.st.slot_pokemon(tp, ts).is_none() {
                    continue;
                }
                let hp = crate::derived::hp(g, tp, ts)?;
                if g.st.slot(tp, ts).damage >= hp {
                    crate::engine::damage::keep_10(g, *t, hp);
                }
            }
            crate::engine::lasting::rearm_takes_more_after_attack(&mut g.st, opp as usize);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn end_turn(g: &mut Game, p: usize) -> R {
    // The turn's records are written below outside any event (no longer the EndTurn reducer since batch 8 C5b): a new
    // version before them and before the state check reads the facts.
    g.st.bump();
    // usedTurnSkip / pendingEndOfTurnEffects: not modeled.
    for s in g.st.players[p].in_play() {
        let slot = &mut g.st.players[p].slots[s as usize];
        slot.healed_this_turn = false;
        if let Some(c) = g.st.slot_pokemon(p, s) {
            g.st.cards[c as usize].damage_taken_last_turn = 0;
        }
    }
    g.st.players[p].pokemon_knocked_out_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_last_turn_entries.clear();
    g.st.players[p].pokemon_knocked_out_last_turn_by_attack.clear();

    // The lasting effects' timing (`engine::lasting::end_turn`).
    crate::engine::lasting::end_turn(&mut g.st, p);
    let pl = &mut g.st.players[p];
    // The turn's record of the Trainer cards played from the hand ends with it.
    pl.played_this_turn.clear();
    let a = pl.active;
    pl.slots[a as usize].attacks_this_turn = Some(0);
    pl.prizes_taken_last_turn = pl.prizes_taken_this_turn;
    pl.prizes_taken_this_turn = 0;
    g.st.bump();
    crate::engine::knockout::state_check(g, OnComplete::AfterEndTurn { p: p as u8 })
}
