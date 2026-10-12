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

    if g.st.turn == 1 && !g.st.rules.first_turn_draw_card {
        g.st.phase = GamePhase::PlayerTurn;
        g.st.bump();
        return Ok(());
    }
    g.st.phase = GamePhase::Draw;
    g.st.bump();
    if g.st.players[p].deck.is_empty() {
        let winner = if g.st.active_player != 0 { WINNER_P1 } else { WINNER_P2 };
        crate::engine::knockout::game_end(g, winner, crate::engine::knockout::EndReason::DeckOut);
        return Ok(());
    }
    // The BeginTurn event (batch 8: no card reacts to it in the pool).
    g.run_fx_unit(Effect::BeginTurn { p: p as u8 })?;

    let id = g.player_id(p);
    // The turn's draw (APR H, G step 4): a Draw event by the rule.
    crate::engine::cards_zone::draw(g, p, 1, crate::cause::Cause::rule(crate::cause::RuleWhich::TurnDraw, p as u8))?;
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
        g.st.bump();
        let id = g.player_id(g.st.active_player as usize);
        g.wait(id, Cont::BetweenTurnsWait { oc });
        return Ok(());
    }
    run_between_turns_effects(g, oc)
}

pub fn run_between_turns_effects(g: &mut Game, oc: OnComplete) -> R {
    // APR F step 2 (E-08, id2302): two or more Checkup effects that apply are ordered by the player whose turn would be
    // next, each before or after the Special Conditions check (batch 8 C11c); otherwise the order of today.
    let mut held = 0;
    for p in 0..2 {
        let a = g.st.players[p].active;
        let slot = g.st.slot(p, a);
        let id = g.new_fx(Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage });
        let t = crate::spec::run::timed_triggers(g, id, crate::effects::k::BETWEEN_TURNS);
        g.release_fx(id);
        held += t?.iter().filter(|(_, x)| *x).count();
    }
    if held >= 2 {
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
    let mut left: SVec<crate::spec::run::Timed, 8> = SVec::new();
    g.checkup_held = true;
    let r = (|| -> R {
        for p in 0..2 {
            let a = g.st.players[p].active;
            let slot = g.st.slot(p, a);
            let id = g.new_fx(Effect::BetweenTurns { p: p as u8, poison_damage: slot.poison_damage, burn_damage: slot.burn_damage });
            ids[p] = id;
            checkup_damage(g, id)?;
            for (t, x) in crate::spec::run::timed_triggers(g, id, crate::effects::k::BETWEEN_TURNS)?.iter().copied() {
                if x && left.len() < left.capacity() {
                    left.push(t);
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
    let chooser = 1 - g.st.active_player as usize;
    order_prompt(g, chooser, left, true, crate::game::OrderThen::Checkup { oc, poison, burn, e: ids });
    Ok(())
}

/// `oc` for EndTurn's checkState, after KO resolution.
pub fn after_end_turn(g: &mut Game, p: usize) -> R {
    // Scenario `expect` assertions at "turn_end": Knock Outs are done, Pokémon Checkup is next.
    crate::expect::on_turn_end(g);
    // Expire KO-time effects on the opponent (denyPrizes: not modeled): Little Grudge.
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

/// The "put N more damage counters on each Poisoned / Burned Pokémon during Pokémon Checkup" declarations
/// (`Modifier::CheckupDamage`: Perilous Jungle, Pecharunt, Magmortar), in the propagation order of the declaring cards (any
/// zone: each checks where its card is), on the Checkup event `id` of a player before it is made.
fn checkup_damage(g: &mut Game, id: EffId) -> R {
    use crate::spec::passive::Modifier;
    let order = g.decl_all(crate::decl::fam::CHECKUP_DAMAGE, 0);
    if order.is_empty() {
        return Ok(());
    }
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

/// Ask `chooser` which of the effects left (and the Special Conditions check, `sc`) applies next (APR E-21: the player
/// whose turn is ending; F step 2: the player whose turn would be next).
pub fn order_prompt(g: &mut Game, chooser: usize, left: SVec<crate::spec::run::Timed, 8>, sc: bool, then: crate::game::OrderThen) {
    let mut names: SVec<&'static str, 8> = SVec::new();
    for t in left.iter() {
        names.push(g.st.cdef(t.c).full_name);
    }
    if sc && names.len() < names.capacity() {
        names.push("Special Conditions");
    }
    let id = g.player_id(chooser);
    g.prompt(
        id,
        "CHOOSE_EFFECT_ORDER",
        PromptKind::Select { values: crate::prompts::SelectValues::Dyn(names), allow_cancel: false, default_value: 0 },
        Cont::OrderEffects { left, sc, then },
    );
}

/// The choice of the order prompt: that effect applies now (its own prompts are answered before the next choice).
pub fn ordered_effect(g: &mut Game, mut left: SVec<crate::spec::run::Timed, 8>, mut sc: bool, then: crate::game::OrderThen, choice: i32) -> R {
    let k = choice.max(0) as usize;
    if k < left.len() {
        let t = left.remove_at(k);
        crate::spec::run::fire_timed(g, t)?;
    } else if sc {
        sc = false;
        checkup_conditions(g, then)?;
    }
    order_next(g, left, sc, then)
}

/// After an ordered effect: once its prompts are answered, the next choice, or the last effect and what follows.
pub fn order_next(g: &mut Game, left: SVec<crate::spec::run::Timed, 8>, sc: bool, then: crate::game::OrderThen) -> R {
    if g.has_prompts() {
        g.wait_prompt(Cont::OrderNext { left, sc, then });
        return Ok(());
    }
    let n = left.len() + sc as usize;
    if n >= 2 {
        let chooser = match then {
            crate::game::OrderThen::EndTurn { p, .. } => p as usize,
            crate::game::OrderThen::Checkup { .. } => 1 - g.st.active_player as usize,
        };
        order_prompt(g, chooser, left, sc, then);
        return Ok(());
    }
    if n == 1 {
        // The last one: no choice left.
        return ordered_effect(g, left, sc, then, 0);
    }
    match then {
        crate::game::OrderThen::EndTurn { p, e } => {
            g.release_fx(e);
            end_turn(g, p as usize)
        }
        crate::game::OrderThen::Checkup { oc, e, .. } => {
            g.release_fx(e[0]);
            g.release_fx(e[1]);
            checkup_finish(g, oc)
        }
    }
}

/// The Special Conditions check of Pokémon Checkup for both players, with their amounts.
fn checkup_conditions(g: &mut Game, then: crate::game::OrderThen) -> R {
    if let crate::game::OrderThen::Checkup { poison, burn, .. } = then {
        for p in 0..2 {
            handle_special_conditions(g, p, poison[p], burn[p])?;
        }
    }
    Ok(())
}

/// After Pokémon Checkup: the state check, once the prompts it opened are answered.
fn checkup_finish(g: &mut Game, oc: OnComplete) -> R {
    if g.has_prompts() {
        g.wait_prompt(Cont::BetweenTurnsCheck { oc });
        return Ok(());
    }
    crate::engine::knockout::state_check(g, oc)
}

/// The EndTurn event's consequence: the once-per-turn marks of player `p`'s Abilities (the cards with a once-per-turn
/// Ability in a zone of the board, `DeclIndex::once_per_turn`) are cleared, in no particular order.
fn end_turn_marks(g: &mut Game, p: usize) {
    let mut set = g.decls.once_per_turn;
    while set != 0 {
        let c = set.trailing_zeros() as CardId;
        set &= set - 1;
        if !crate::decl::in_a_zone(g, c) {
            continue;
        }
        let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { continue };
        for pw in spec.powers.iter() {
            if let crate::spec::Once::PerTurn(name) | crate::spec::Once::PerTurnShared(name) = pw.once {
                let m = &mut g.st.players[p].marker;
                let id = crate::markers::intern(name);
                if m.has_from(id, c) {
                    m.remove_from(id, c);
                }
            }
        }
    }
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::EndTurn { p } => {
            end_turn_marks(g, p as usize);
            Ok(())
        }
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

    // The lasting effects' timing (`engine::lasting::end_turn`).
    crate::engine::lasting::end_turn(&mut g.st, p);
    tick_play_locks_at_end_of_turn(&mut g.st.players[p]);
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

/// The player's turn counters ("Stadium and Tool cards have no effect", unlimited Energy attachments) count this turn.
pub(crate) fn tick_play_locks_at_end_of_turn(pl: &mut Player) {
    if pl.stadium_and_tool_have_no_effect_turns_remaining > 0 {
        pl.stadium_and_tool_have_no_effect_turns_remaining -= 1;
    }
    if pl.unlimited_energy_attach_turns_remaining > 0 {
        pl.unlimited_energy_attach_turns_remaining -= 1;
    }
    pl.used_dragons_wish = pl.unlimited_energy_attach_turns_remaining == 1;
}

pub fn _unused(_: SVec<u8, 1>) {}
