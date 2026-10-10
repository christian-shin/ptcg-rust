//! Turn structure (`game-phase-effect.ts`): draw, end of turn, between
//! turns, special conditions.
//!
//! Fields Twinleaf clears here that no pool card ever sets are not modeled;
//! they are listed next to the code that would touch them.

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
    g.st.winner = winner;
    g.st.phase = GamePhase::Finished;
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

    if g.st.turn == 1 && !g.st.rules.first_turn_draw_card {
        g.st.phase = GamePhase::PlayerTurn;
        return Ok(());
    }
    g.st.phase = GamePhase::Draw;
    if g.st.players[p].deck.is_empty() {
        let winner = if g.st.active_player != 0 { WINNER_P1 } else { WINNER_P2 };
        crate::engine::knockout::game_end(g, winner, crate::engine::knockout::EndReason::DeckOut);
        return Ok(());
    }
    g.run_fx_unit(Effect::BeginTurn { p: p as u8 })?;

    let id = g.player_id(p);
    if g.st.players[p].cannot_draw_at_start_of_turn {
        g.st.players[p].cannot_draw_at_start_of_turn = false;
    } else {
        // The turn's draw (APR H, G step 4): a Draw event by the rule.
        crate::engine::cards_zone::draw(g, p, 1, crate::cause::Cause::rule(crate::cause::RuleWhich::TurnDraw, p as u8))?;
    }
    g.wait(id, Cont::PhasePlayerTurn);
    Ok(())
}

fn start_next_turn(g: &mut Game) -> R {
    let p = g.st.active_player as usize;
    let a = g.st.players[p].active;
    // Pokémon Checkup: a Pokémon Paralyzed since the start of its owner's last turn recovers.
    crate::engine::condition::remove(g, SlotRef::new(p, a), SpecialCondition::Paralyzed, crate::engine::condition::by_checkup(p))?;
    g.move_to(ListRef::Supporter(p as u8), ListRef::Discard(p as u8), None);
    between_turns(g, OnComplete::InitNextTurn)
}

pub fn between_turns(g: &mut Game, oc: OnComplete) -> R {
    let entered = g.st.phase == GamePhase::PlayerTurn || g.st.phase == GamePhase::Attack;
    if entered {
        g.st.phase = GamePhase::BetweenTurns;
        let id = g.player_id(g.st.active_player as usize);
        g.wait(id, Cont::BetweenTurnsWait { oc });
        return Ok(());
    }
    run_between_turns_effects(g, oc)
}

pub fn run_between_turns_effects(g: &mut Game, oc: OnComplete) -> R {
    for p in 0..2 {
        let a = g.st.players[p].active;
        let slot = g.st.slot(p, a);
        let e = Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage };
        g.run_fx_unit(e)?;
    }
    if g.has_prompts() {
        g.wait_prompt(Cont::BetweenTurnsCheck { oc });
        return Ok(());
    }
    crate::engine::knockout::state_check(g, oc)
}

/// `oc` for EndTurn's checkState, after KO resolution.
pub fn after_end_turn(g: &mut Game, p: usize) -> R {
    // Scenario `expect` assertions at "turn_end": Knock Outs are done, Pokémon Checkup is next.
    crate::expect::on_turn_end(g);
    // Expire KO-time effects on the opponent (denyPrizes: not modeled).
    let o = 1 - p;
    for s in g.st.players[o].in_play().iter() {
        if g.st.slot_pokemon(o, *s).is_none() {
            continue;
        }
        let slot = &mut g.st.players[o].slots[*s as usize];
        slot.discard_attacker_energy_if_ko_next_turn = false;
        slot.discard_attacker_energy_if_ko_next_turn_pending = false;
        slot.discard_attacker_energy_if_ko_attack = None;
        slot.discard_attacker_energy_if_ko_source_card = None;
        slot.discard_attacker_energy_if_ko_attacker = None;
    }
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

/// `WOULD_CHANGE_SPECIAL_CONDITIONS` (prefabs/special-condition-change.ts): would making the slot's Pokémon
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

fn handle_special_conditions(g: &mut Game, id: EffId) -> R {
    let (p, poison, burn) = match *g.e(id) {
        Effect::BetweenTurns { p, poison_damage, burn_damage } => (p as usize, poison_damage, burn_damage),
        _ => return Ok(()),
    };
    let pid = g.player_id(p);
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
                g.prompt(pid, "FLIP_BURNED", PromptKind::CoinFlip, Cont::BurnFlip { p: p as u8, slot: a });
            }
            SpecialCondition::Asleep => {
                let flips = g.st.slot(p, a).sleep_flips.max(0) as usize;
                if flips > 0 {
                    let prompts: Vec<(u8, &'static str, PromptKind)> = (0..flips).map(|_| (pid, "FLIP_ASLEEP", PromptKind::CoinFlip)).collect();
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

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::EndTurn { p } => end_turn(g, p as usize),
        Effect::AfterAttack { opp, .. } => {
            // A Pokémon that survived this attack on 10 HP keeps 10 HP when an effect of the
            // attack lowered its maximum HP (survive-on-ten.ts; ruling 1589).
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
            let o = opp as usize;
            for s in g.st.players[o].in_play().iter() {
                let slot = &mut g.st.players[o].slots[*s as usize];
                if slot.defending_extra_damage_rearm_after_attack {
                    slot.defending_extra_damage_rearm_after_attack = false;
                    slot.defending_extra_damage_pending = true;
                }
            }
            Ok(())
        }
        Effect::BetweenTurns { .. } => handle_special_conditions(g, id),
        _ => Ok(()),
    }
}

fn end_turn(g: &mut Game, p: usize) -> R {
    let o = 1 - p;
    // Only an attack used during this turn counts (phase 4b: a turn that ended
    // without an attack kept the previous attack's flag).
    let ancient = match g.st.player_last_attack[p] {
        Some((_, src)) => g.st.player_last_attack_turn[p] == g.st.turn && g.st.cdef(src).has_tag(tag::ANCIENT),
        None => false,
    };
    g.st.players[p].ancient_pokemon_attacked_last_turn = ancient;
    // usedTurnSkip / pendingEndOfTurnEffects: not modeled.
    g.st.players[p].can_evolve = false;
    for s in g.st.players[p].in_play().iter() {
        let slot = &mut g.st.players[p].slots[*s as usize];
        slot.board_effect.retain(|b| *b != BoardEffect::AbilityUsed as u8);
        slot.healed_this_turn = false;
        if let Some(c) = g.st.slot_pokemon(p, *s) {
            g.st.cards[c as usize].damage_taken_last_turn = 0;
        }
    }
    g.st.players[p].pokemon_knocked_out_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_last_turn_entries.clear();
    g.st.players[p].pokemon_knocked_out_last_turn_by_attack.clear();

    for s in g.st.players[o].in_play().iter() {
        let slot = &mut g.st.players[o].slots[*s as usize];
        slot.damage_reduction_next_turn = 0;
        slot.no_weakness_next_turn = false;
        slot.retaliate_on_damage_next_turn = None;
        // The preventions the opponent's attack left on its Pokémon end with this turn (armed during its turn: pending
        // ones too).
        slot.lasting_prevents.clear();
    }
    for s in g.st.players[p].in_play().iter() {
        let slot = &mut g.st.players[p].slots[*s as usize];
        // The locks an attack left on the Pokémon count this turn of its owner (events batch 7, D14).
        for l in slot.lasting_locks.as_mut_slice().iter_mut() {
            if l.turns_remaining != LastingLock::UNTIL_REMOVED {
                l.turns_remaining -= 1;
            }
        }
        slot.lasting_locks.retain(|l| l.turns_remaining > 0);
        // A prevention armed during this turn is in force during the opponent's next turn.
        for l in slot.lasting_prevents.as_mut_slice().iter_mut() {
            l.pending = false;
        }
        if slot.no_weakness_next_turn_pending {
            slot.no_weakness_next_turn = true;
            slot.no_weakness_next_turn_pending = false;
        }
        if slot.retaliate_on_damage_next_turn_pending.is_some() {
            slot.retaliate_on_damage_next_turn = slot.retaliate_on_damage_next_turn_pending.take();
        }
        if slot.discard_attacker_energy_if_ko_next_turn_pending {
            slot.discard_attacker_energy_if_ko_next_turn = true;
            slot.discard_attacker_energy_if_ko_next_turn_pending = false;
        }
        if slot.attack_damage_reduction_next_turn > 0 {
            slot.attack_damage_reduction_next_turn = 0;
        }
        // Replace the previous bonus with one armed during this turn, or clear it.
        slot.next_turn_attack_damage_bonus = slot.next_turn_attack_damage_bonus_pending;
        slot.next_turn_attack_damage_bonus_pending = None;
    }
    // defendingPokemonExtraDamage*: arm at the end of the defending player's
    // turn, then clear at the end of the attacker's following turn.
    for q in [p, o] {
        for s in g.st.players[q].in_play().iter() {
            let slot = &mut g.st.players[q].slots[*s as usize];
            if slot.defending_extra_damage_pending && slot.defending_extra_damage_attacker != Some(p as u8) {
                slot.defending_extra_damage_pending = false;
            }
        }
    }
    for q in [p, o] {
        for s in g.st.players[q].in_play().iter() {
            let slot = &mut g.st.players[q].slots[*s as usize];
            if slot.defending_extra_damage_attacker == Some(p as u8) && !slot.defending_extra_damage_pending && slot.defending_extra_damage_next_turn > 0 {
                slot.defending_extra_damage_next_turn = 0;
                slot.defending_extra_damage_attacker = None;
            }
        }
    }
    cost_increase_end_of_turn(g, p);
    tick_play_locks_at_end_of_turn(&mut g.st.players[p]);
    let pl = &mut g.st.players[p];
    // The turn's record of the Trainer cards played from the hand ends with it.
    pl.played_this_turn.clear();
    let a = pl.active;
    pl.slots[a as usize].attacks_this_turn = Some(0);
    pl.prizes_taken_last_turn = pl.prizes_taken_this_turn;
    pl.prizes_taken_this_turn = 0;
    crate::engine::knockout::state_check(g, OnComplete::AfterEndTurn { p: p as u8 })
}

/// EndTurnEffect: arm / expire `attackCostIncreaseNextTurn` and
/// `retreatCostIncreaseNextTurn` on both players' Pokémon (`p` ends its turn).
/// Phase 4b (R6): armed when the attacker ends its turn (live during the
/// defender's turn) and expired when the defender ends its turn; they used to
/// be armed at the defender's end of turn, i.e. live in the attacker's next
/// turn, where they never bit.
fn cost_increase_end_of_turn(g: &mut Game, p: usize) {
    let me = Some(p as u8);
    for pass in 0..2 {
        for q in [p, 1 - p] {
            for s in g.st.players[q].in_play().iter() {
                if g.st.slot_pokemon(q, *s).is_none() {
                    continue;
                }
                let slot = &mut g.st.players[q].slots[*s as usize];
                if pass == 0 {
                    if slot.attack_cost_increase_next_turn_pending != 0 && slot.attack_cost_increase_next_turn_attacker == me {
                        slot.attack_cost_increase_next_turn = slot.attack_cost_increase_next_turn_pending;
                        slot.attack_cost_increase_next_turn_pending = 0;
                    }
                    if slot.retreat_cost_increase_next_turn_pending != 0 && slot.retreat_cost_increase_next_turn_attacker == me {
                        slot.retreat_cost_increase_next_turn = slot.retreat_cost_increase_next_turn_pending;
                        slot.retreat_cost_increase_next_turn_pending = 0;
                    }
                } else {
                    if slot.attack_cost_increase_next_turn_attacker.is_some()
                        && slot.attack_cost_increase_next_turn_attacker != me
                        && slot.attack_cost_increase_next_turn_pending == 0
                        && slot.attack_cost_increase_next_turn > 0
                    {
                        slot.attack_cost_increase_next_turn = 0;
                        slot.attack_cost_increase_next_turn_attacker = None;
                    }
                    if slot.retreat_cost_increase_next_turn_attacker.is_some()
                        && slot.retreat_cost_increase_next_turn_attacker != me
                        && slot.retreat_cost_increase_next_turn_pending == 0
                        && slot.retreat_cost_increase_next_turn > 0
                    {
                        slot.retreat_cost_increase_next_turn = 0;
                        slot.retreat_cost_increase_next_turn_attacker = None;
                    }
                }
            }
        }
    }
}

/// A lock an attack leaves on the player for `turns_remaining` of their turns (a lock already there, the same
/// declaration, keeps the longer).
pub fn apply_play_lock(pl: &mut Player, lock: &'static crate::spec::passive::LockDecl, turns_remaining: i8, source: crate::list::CardId) {
    let turns = turns_remaining.max(1);
    if let Some(l) = pl.lasting_locks.iter_mut().flatten().find(|l| l.decl.same_as(lock)) {
        l.turns_remaining = l.turns_remaining.max(turns);
        return;
    }
    let slot = pl.lasting_locks.iter().position(|l| l.is_none()).unwrap_or(pl.lasting_locks.len() - 1);
    pl.lasting_locks[slot] = Some(LastingLock { decl: lock, turns_remaining: turns, source, attack: None });
}

pub(crate) fn tick_play_locks_at_end_of_turn(pl: &mut Player) {
    for slot in pl.lasting_locks.iter_mut() {
        if let Some(l) = slot {
            l.turns_remaining -= 1;
            if l.turns_remaining <= 0 {
                *slot = None;
            }
        }
    }
    if pl.stadium_and_tool_have_no_effect_turns_remaining > 0 {
        pl.stadium_and_tool_have_no_effect_turns_remaining -= 1;
    }
    if pl.unlimited_energy_attach_turns_remaining > 0 {
        pl.unlimited_energy_attach_turns_remaining -= 1;
    }
    pl.used_dragons_wish = pl.unlimited_energy_attach_turns_remaining == 1;
    pl.cannot_draw_at_start_of_turn = false;
}

pub fn _unused(_: SVec<u8, 1>) {}
