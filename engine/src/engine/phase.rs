//! Turn structure (`game-phase-effect.ts`): draw, end of turn, between
//! turns, special conditions.
//!
//! Fields Twinleaf clears here that no pool card ever sets are not modeled;
//! they are listed next to the code that would touch them.

use crate::effects::*;
use crate::engine::check;
use crate::game::{Cont, Game, OnComplete, R};
use crate::list::*;
use crate::markers::*;
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
        end_game(g, winner);
        return Ok(());
    }
    g.run_fx(Effect::BeginTurn { p: p as u8 })?;

    let id = g.player_id(p);
    let draw = if g.st.players[p].cannot_draw_at_start_of_turn {
        g.st.players[p].cannot_draw_at_start_of_turn = false;
        None
    } else {
        match g.run_fx(Effect::DrawCardForTurn { p: p as u8, draw_count: 1 }) {
            Ok((Effect::DrawCardForTurn { draw_count, .. }, _)) => Some(draw_count),
            _ => None,
        }
    };
    let draw_count = match draw {
        Some(n) => n,
        None => {
            g.wait(id, Cont::PhasePlayerTurn);
            return Ok(());
        }
    };
    let hand_start = g.st.players[p].hand.len();
    g.run_fx(Effect::MoveCards {
        source: ListRef::Deck(p as u8),
        destination: ListRef::Hand(p as u8),
        cards: None,
        count: Some(draw_count),
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: NO_CARD,
    })?;
    let drawn = g.st.players[p].hand.len().saturating_sub(hand_start);
    for i in 0..drawn {
        let card = g.st.players[p].hand.as_slice()[hand_start + i];
        if g.run_fx(Effect::DrewTopdeck { p: p as u8, card }).is_err() {
            g.wait(id, Cont::PhasePlayerTurn);
            return Ok(());
        }
    }
    g.wait(id, Cont::PhasePlayerTurn);
    Ok(())
}

fn start_next_turn(g: &mut Game) -> R {
    let p = g.st.active_player as usize;
    let a = g.st.players[p].active;
    remove_condition(g, p, a, SpecialCondition::Paralyzed);
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
        let e = Effect::BetweenTurns {
            p: p as u8,
            poison_damage: slot.poison_damage,
            burn_damage: slot.burn_damage,
            burn_flip_result: None,
            asleep_flip_result: None,
        };
        g.run_fx(e)?;
    }
    if g.has_prompts() {
        g.wait_prompt(Cont::BetweenTurnsCheck { oc });
        return Ok(());
    }
    check::check_state(g, oc)
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

pub fn remove_condition(g: &mut Game, p: usize, s: SlotId, sc: SpecialCondition) {
    let v = sc as u8;
    let conds = &mut g.st.players[p].slots[s as usize].special_conditions;
    if conds.contains(&v) {
        conds.retain(|x| *x != v);
    }
}

pub fn remove_active_condition(g: &mut Game, p: usize, sc: SpecialCondition) {
    let a = g.st.players[p].active;
    remove_condition(g, p, a, sc);
}

pub fn add_condition(slot: &mut Slot, sc: SpecialCondition) {
    // cannotBeSpecialConditionedNextTurn: not modeled.
    match sc {
        SpecialCondition::Poisoned => slot.poison_damage = 10,
        SpecialCondition::Burned => slot.burn_damage = 20,
        SpecialCondition::Confused => slot.confusion_damage = 30,
        _ => {}
    }
    let v = sc as u8;
    if slot.special_conditions.contains(&v) {
        return;
    }
    if sc == SpecialCondition::Poisoned || sc == SpecialCondition::Burned {
        slot.special_conditions.push(v);
        return;
    }
    slot.special_conditions.retain(|s| {
        !(*s == SpecialCondition::Paralyzed as u8 || *s == SpecialCondition::Confused as u8 || *s == SpecialCondition::Asleep as u8)
    });
    slot.special_conditions.push(v);
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

fn handle_special_conditions(g: &mut Game, id: EffId) {
    let (p, poison, burn, burn_flip, asleep_flip) = match *g.e(id) {
        Effect::BetweenTurns { p, poison_damage, burn_damage, burn_flip_result, asleep_flip_result } => {
            (p as usize, poison_damage, burn_damage, burn_flip_result, asleep_flip_result)
        }
        _ => return,
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
        match SpecialCondition::from_u8(sp) {
            SpecialCondition::Poisoned => g.st.players[p].slots[a as usize].damage += poison,
            SpecialCondition::Burned => {
                g.st.players[p].slots[a as usize].damage += burn;
                match burn_flip {
                    Some(true) => {}
                    Some(false) => g.st.players[p].slots[a as usize].damage += burn,
                    None => g.prompt(pid, "FLIP_BURNED", PromptKind::CoinFlip, Cont::BurnFlip { p: p as u8, slot: a }),
                }
            }
            SpecialCondition::Asleep => match asleep_flip {
                Some(true) => remove_active_condition(g, p, SpecialCondition::Asleep),
                Some(false) => {}
                None => {
                    let flips = g.st.slot(p, a).sleep_flips.max(0) as usize;
                    if flips > 0 {
                        let prompts: Vec<(u8, &'static str, PromptKind)> =
                            (0..flips).map(|_| (pid, "FLIP_ASLEEP", PromptKind::CoinFlip)).collect();
                        g.prompt_group(&prompts, Cont::SleepFlips { p: p as u8, slot: a });
                    } else {
                        remove_active_condition(g, p, SpecialCondition::Asleep);
                    }
                }
            },
            _ => {}
        }
    }
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
                let hp = crate::engine::check::check_hp(g, tp, ts)?;
                if g.st.slot(tp, ts).damage >= hp {
                    g.st.players[tp].slots[ts as usize].damage = hp - 10;
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
        Effect::BetweenTurns { .. } => {
            handle_special_conditions(g, id);
            Ok(())
        }
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
    g.st.players[p].marker.remove(DAMAGE_DEALT_MARKER);
    g.st.players[p].pokemon_knocked_out_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn = false;
    g.st.players[p].pokemon_knocked_out_last_turn_entries.clear();

    for s in g.st.players[o].in_play().iter() {
        let slot = &mut g.st.players[o].slots[*s as usize];
        slot.damage_reduction_next_turn = 0;
        slot.prevent_damage_next_turn = false;
        slot.prevent_damage_next_turn_pending = false;
        slot.prevent_damage_filter = Default::default();
        slot.prevent_damage_filter_pending = Default::default();
        slot.prevent_effects_of_attacks_next_turn = false;
        slot.prevent_effects_of_attacks_next_turn_pending = false;
        slot.no_weakness_next_turn = false;
        slot.retaliate_on_damage_next_turn = None;
        // other next-turn protections: not modeled.
    }
    for s in g.st.players[p].in_play().iter() {
        // Phase 4b (R3): "This Pokémon can't use [attack]" only locks an attack the Pokémon
        // has; a Pokémon that copied the attack (Slowking's Seek Inspiration, Metronome) used
        // its own attack, so the copy doesn't lock the copied name (Rulings Compendium 1654).
        // TS: `cannotUseAttacksNextTurnPending.filter(name => cards.some(attacks has name))`.
        // Memory Helix (Mew ex) is different: the Pokémon uses the attack itself, so the lock keeps
        // a name that is among the Active Pokémon's offered (copied) attacks.
        let mut owned: SVec<&'static str, 4> = SVec::new();
        let mut copied_names: Option<SVec<&'static str, 32>> = None;
        let pending: SVec<&'static str, 4> = g.st.players[p].slots[*s as usize].cannot_use_attacks_next_turn_pending;
        for n in pending.iter() {
            let sl = &g.st.players[p].slots[*s as usize];
            if sl.cards.iter().any(|c| {
                let d = g.st.cdef(c);
                d.is_pokemon() && d.attacks.iter().any(|x| x.name == *n)
            }) {
                owned.push(*n);
                continue;
            }
            if *s != g.st.players[p].active {
                continue;
            }
            if copied_names.is_none() {
                let mut v: SVec<&'static str, 32> = SVec::new();
                let e = crate::engine::turn::check_attacks_effect(g, p);
                if let (Effect::CheckPokemonAttacks { copied, .. }, _) = g.run_fx(e)? {
                    for a in copied.iter() {
                        v.push(g.st.cdef(a.card).attacks[a.idx()].name);
                    }
                }
                copied_names = Some(v);
            }
            if copied_names.as_ref().map_or(false, |v| v.iter().any(|x| x == n)) {
                owned.push(*n);
            }
        }
        let slot = &mut g.st.players[p].slots[*s as usize];
        if slot.prevent_damage_next_turn_pending {
            slot.prevent_damage_next_turn = true;
            slot.prevent_damage_next_turn_pending = false;
            slot.prevent_damage_filter = slot.prevent_damage_filter_pending;
            slot.prevent_damage_filter_pending = Default::default();
        }
        if slot.prevent_effects_of_attacks_next_turn_pending {
            slot.prevent_effects_of_attacks_next_turn = true;
            slot.prevent_effects_of_attacks_next_turn_pending = false;
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
        if slot.cannot_attack_next_turn {
            slot.cannot_attack_next_turn = false;
        }
        if !slot.cannot_use_attacks_next_turn.is_empty() {
            slot.cannot_use_attacks_next_turn.clear();
        }
        if slot.cannot_attack_next_turn_pending {
            slot.cannot_attack_next_turn = true;
            slot.cannot_attack_next_turn_pending = false;
        }
        if !slot.cannot_use_attacks_next_turn_pending.is_empty() {
            slot.cannot_use_attacks_next_turn = owned;
            slot.cannot_use_attacks_next_turn_pending.clear();
        }
        if slot.attack_damage_reduction_next_turn > 0 {
            slot.attack_damage_reduction_next_turn = 0;
        }
        if slot.cannot_retreat_next_turn {
            slot.cannot_retreat_next_turn = false;
        }
        if slot.cannot_retreat_next_turn_pending {
            slot.cannot_retreat_next_turn = true;
            slot.cannot_retreat_next_turn_pending = false;
        }
        if slot.cannot_be_healed_next_turn {
            slot.cannot_be_healed_next_turn = false;
        }
        slot.blocked_attack_name_next_turn = None;
        if slot.prevent_damage_next_turn_pending {
            slot.prevent_damage_next_turn = true;
            slot.prevent_damage_next_turn_pending = false;
        }
        if slot.prevent_effects_of_attacks_next_turn_pending {
            slot.prevent_effects_of_attacks_next_turn = true;
            slot.prevent_effects_of_attacks_next_turn_pending = false;
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
    pl.supporter_turn = 0;
    pl.rocket_supporter = false;
    let a = pl.active;
    pl.slots[a as usize].attacks_this_turn = Some(0);
    pl.prizes_taken_last_turn = pl.prizes_taken_this_turn;
    pl.prizes_taken_this_turn = 0;
    check::check_state(g, OnComplete::AfterEndTurn { p: p as u8 })
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

pub fn clear_play_locks(pl: &mut Player) {
    pl.cannot_play_item_cards = false;
    pl.cannot_play_supporter_cards = false;
    pl.cannot_play_stadium_cards = false;
    pl.cannot_play_tool_cards = false;
    pl.cannot_play_special_energy_cards = false;
    pl.cannot_play_energy_cards = false;
    pl.cannot_play_pokemon_cards = false;
    pl.cannot_play_pokemon_with_abilities = false;
    pl.cannot_evolve_pokemon_cards = false;
    pl.play_locks_turns_remaining = 0;
}

/// `Player.applyPlayLocks(locks, turnsRemaining)`.
pub fn apply_play_locks(pl: &mut Player, locks: u16, turns_remaining: i32) {
    use crate::effects::play_lock as l;
    if locks & l::ITEM != 0 {
        pl.cannot_play_item_cards = true;
    }
    if locks & l::SUPPORTER != 0 {
        pl.cannot_play_supporter_cards = true;
    }
    if locks & l::STADIUM != 0 {
        pl.cannot_play_stadium_cards = true;
    }
    if locks & l::TOOL != 0 {
        pl.cannot_play_tool_cards = true;
    }
    if locks & l::SPECIAL_ENERGY != 0 {
        pl.cannot_play_special_energy_cards = true;
    }
    if locks & l::ENERGY != 0 {
        pl.cannot_play_energy_cards = true;
        pl.cannot_play_special_energy_cards = true;
    }
    if locks & l::POKEMON != 0 {
        pl.cannot_play_pokemon_cards = true;
    }
    if locks & l::POKEMON_WITH_ABILITIES != 0 {
        pl.cannot_play_pokemon_with_abilities = true;
    }
    if locks & l::EVOLVE != 0 {
        pl.cannot_evolve_pokemon_cards = true;
    }
    pl.play_locks_turns_remaining = pl.play_locks_turns_remaining.max(turns_remaining.max(1));
}

fn tick_play_locks_at_end_of_turn(pl: &mut Player) {
    if pl.play_locks_turns_remaining > 0 {
        pl.play_locks_turns_remaining -= 1;
        if pl.play_locks_turns_remaining <= 0 {
            clear_play_locks(pl);
        }
    }
    if pl.stadium_and_tool_have_no_effect_turns_remaining > 0 {
        pl.stadium_and_tool_have_no_effect_turns_remaining -= 1;
    }
    if pl.coin_flip_cancel_trainer_play_turns_remaining > 0 {
        pl.coin_flip_cancel_trainer_play_turns_remaining -= 1;
    }
    if pl.cannot_attack_turns_remaining > 0 {
        pl.cannot_attack_turns_remaining -= 1;
    }
    if pl.cannot_attack_max_energy_turns_remaining > 0 {
        pl.cannot_attack_max_energy_turns_remaining -= 1;
        if pl.cannot_attack_max_energy_turns_remaining <= 0 {
            pl.cannot_attack_max_energy = None;
        }
    }
    if pl.unlimited_energy_attach_turns_remaining > 0 {
        pl.unlimited_energy_attach_turns_remaining -= 1;
    }
    pl.used_dragons_wish = pl.unlimited_energy_attach_turns_remaining == 1;
    pl.cannot_draw_at_start_of_turn = false;
}

pub fn _unused(_: SVec<u8, 1>) {}
