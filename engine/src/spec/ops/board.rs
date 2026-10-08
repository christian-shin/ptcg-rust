//! Damage, damage counters, healing, switching, Special Conditions, Knock
//! Outs, evolution, removing Pokémon from play (vocabulary v1 "Operations",
//! board and slots).
//!
//! Slot choices are part of the op that uses them (`SlotTarget::Pick`): there
//! are no slot registers in the interpreter yet, so a choice cannot be shared
//! between two ops. An attack's choice is asked at step D (`choice`) and read
//! again after the damage (`exec`).
//!
//! The ops evaluate their numbers and guards with `num_m` / `cond_m`
//! (checked reads: Energy provided, types as the game checks them).

use super::super::run::{Flow, Frame, Phase, CHOICE_NONE, CHOICE_YES, NONE};
use super::super::*;
use crate::effects::{AtkBase, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::prompts::*;
use crate::state::{ListRef, SlotId};
use crate::types::*;

/// Card files name Special Conditions and Pokémon types through the prelude.
pub use crate::types::{ct, SpecialCondition};

/// A prompt asking `chooser` to pick one Pokémon among `among`.
pub struct PickSlotSpec {
    pub chooser: Who,
    pub among: SlotSel,
    pub msg: &'static str,
}

/// The Pokémon an op acts on: fixed, or picked by a player.
pub enum SlotTarget {
    Slot(SlotExpr),
    Pick(PickSlotSpec),
}

/// How a switch is carried out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwitchKind {
    /// `Player.switchPokemon(target, store, state)`: movement effects fire.
    Plain,
    /// `switchPokemon(target)` without dispatching movement effects.
    Silent,
    /// `Plain`, with only the side's Benched Basic Pokémon to choose from (S3).
    PlainBasic,
    /// An effect of this card's Ability (EffectOfAbility probe, power 0), then `Silent` if the target
    /// survives it (Sumo Catcher).
    SilentAbilityEffect,
    /// Gust: a GustOpponentBenchEffect (preventable by attack effect protection).
    Gust,
    /// Switch out the opponent's Active: a SwitchOutOpponentsActiveEffect probe
    /// before the new Active is chosen, and again with it.
    SwitchOut,
    // --- S3 agent 3 appends ---
    /// The Pokémon chosen by `PickSlot` (a Benched Pokémon of `side`) becomes Active, silently,
    /// without asking.
    Picked,
}

/// "Switch": the Pokémon in the Active Spot of `side` changes places with a
/// Benched Pokémon chosen by `chooser`.
pub struct SwitchSpec {
    pub side: Who,
    pub chooser: Who,
    pub kind: SwitchKind,
    pub msg: &'static str,
    /// A Trainer or Ability can't be used without a Benched Pokémon to switch
    /// with (otherwise the switch is simply skipped).
    pub required: bool,
}

/// Which effect carries out a heal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HealVia {
    /// `HealTargetEffect`, built on the attack.
    Attack,
    /// `HealEffect`.
    Effect,
}

pub struct HealSpec {
    pub target: SlotTarget,
    pub hp: Num,
    pub via: HealVia,
    /// Also remove every Special Condition from the Pokémon (after the heal).
    pub clear_conditions: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageOp {
    /// "N more damage".
    Add,
    /// "N times": the attack's damage is exactly N.
    Set,
}

/// Change the damage the attack is about to do (before the damage).
pub struct DamageSpec {
    pub op: DamageOp,
    pub hp: Num,
    pub when: Cond,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageCalc {
    /// A DealDamageEffect on the opponent's Active, a PutDamageEffect elsewhere.
    Auto,
    /// A DealDamageEffect (Weakness and Resistance apply).
    Deal,
    /// A PutDamageEffect (no Weakness or Resistance).
    Put,
    // --- S3 agent 3 appends ---
    /// The damage is written on the Pokémon directly (no effect, no Knock Out check).
    Direct,
}

/// Damage the attack does to a Pokémon other than the Defending one (or to
/// itself).
pub struct DamageSlotSpec {
    pub target: SlotTarget,
    pub hp: Num,
    /// Plus this many times the damage (in HP) on the target when it is hit.
    pub target_damage_mul: i32,
    pub calc: DamageCalc,
    pub when: Cond,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CounterCause {
    /// PutCountersEffect, built on the attack.
    Attack,
    /// PlaceDamageCountersEffect (Ability or card effect).
    Effect,
    /// Written on the slot without an effect (nothing can prevent it); the slot can still be
    /// empty (a Pokémon about to be put there).
    Direct,
}

pub struct PlaceCountersSpec {
    pub target: SlotTarget,
    pub counters: Num,
    pub cause: CounterCause,
}
/// Put up to `total_hp` damage, in counters, on the chooser's Active Pokémon
/// (the allocation prompt lists every Pokémon in play, each capped at its HP
/// plus `cap_bonus_hp`); the attack then does `damage_per_hp` damage for each
/// HP of counters placed (the last entry decides).
pub struct SpreadCountersSpec {
    pub chooser: Who,
    pub total_hp: i32,
    pub cap_bonus_hp: i32,
    pub damage_per_hp: i32,
}

pub enum MoveCountersKind {
    /// All the damage counters on one chosen Pokémon move to another chosen
    /// Pokémon. Both are asked at step D for an attack.
    AllFromOne { from: PickSlotSpec, to: PickSlotSpec },
    /// Any number of counters move between the Pokémon of one side.
    AnyAmong { who: Who },
    // --- S3 agent 3 appends ---
    /// Up to `max` damage counters move from 1 of the player's Pokémon to 1 of the opponent's
    /// (Munkidori's Adrena-Brain); the counters move one at a time.
    MineToOpp { max: u8 },
}

pub struct MoveCountersSpec {
    pub kind: MoveCountersKind,
}
/// Evolve a Pokémon.
pub struct EvolveSpec {
    pub how: EvolveHow,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvolveHow {
    /// Rare Candy: a Basic Pokémon (in play before this turn) is evolved into the Stage 2 card of
    /// its line in the hand, skipping the Stage 1; the player chooses the Pokémon, then the card
    /// (`Cond::RareCandyUsable` says whether it can be done).
    RareCandy,
    /// Put the card of register `card` onto the Pokémon as an evolution by an effect (no Evolve
    /// effect: the slot loses its effects and counts as played this turn).
    PutOnto { slot: SlotExpr, card: u8 },
    /// Grand Tree: a Pokémon evolves with a card from the deck (a ChoosePokemon prompt over the Basic Pokémon
    /// that can evolve now, then a ChooseCards prompt over the deck for a `stage` card that evolves from it,
    /// cancellable); with `then_stage` a second card evolving from the first is offered. The deck is not
    /// shuffled here.
    FromDeck { chooser: Who, stage: Stage, then_stage: Option<Stage> },
}
/// Devolve the Pokémon (when it has an evolution card): the top card goes to `destination`.
/// An effect of the attack that effect protection can stop.
pub struct DevolveSpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
}
/// Put the Pokémon card in card register `cards` onto this card's Pokémon (as it is, evolution state
/// kept) and this card into `into`.
pub struct SwapPokemonCardSpec {
    pub cards: u8,
    /// The Pokémon whose top card is replaced.
    pub slot: SlotExpr,
    pub into: ZoneRef,
    /// The new card takes the old card's place in the stack.
    pub keep_index: bool,
}
/// This Pokémon (the slot `target`) switches with the Active Pokémon when it is on the Bench.
pub struct SwitchWithActiveSpec {
    pub target: SlotExpr,
}
/// Put a Pokémon and all cards attached to it into a zone.
pub struct RemoveFromPlaySpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
    /// An effect of the attack being used: effect protection on the Pokémon (Mist Energy and the like) stops it.
    pub effect_of_attack: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cause {
    /// An effect of the attack (AddSpecialConditionsEffect): effect protection applies.
    Attack,
    /// An Ability-style effect (AddSpecialConditionsPowerEffect), also what
    /// resets the Poison, Burn and Confusion values to the defaults.
    Ability,
    /// Written directly on the Pokémon.
    Direct,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConditionChange {
    Add(&'static [SpecialCondition]),
    RemoveAll,
    /// Remove one: the only one, or the one the player picks.
    RemoveChosen,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    None,
    /// A TrainerTargetEffect on the Pokémon must not be blocked.
    TrainerTarget,
}

pub struct ConditionsSpec {
    pub target: SlotExpr,
    pub change: ConditionChange,
    pub cause: Cause,
    pub gate: Gate,
    pub when: Cond,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KnockOutMode {
    /// A KnockOutOpponentEffect: the attacker takes the Prizes.
    Opponent,
    /// A KnockOutPlayerEffect: the opponent takes the Prizes.
    Player,
    /// The Pokémon is Knocked Out at the next check (`damage += 999`).
    Direct,
}

pub struct KnockOutSpec {
    pub target: SlotExpr,
    pub mode: KnockOutMode,
    pub when: Cond,
}

/// The attacker picks `count` of the Pokémon in `among` (all of them when fewer; none when none),
/// asked at step D, and the attack does `hp` damage to each, in the order they were picked.
pub struct DamageChosenSpec {
    pub among: SlotSel,
    pub count: i32,
    pub hp: Num,
    pub calc: DamageCalc,
    pub msg: &'static str,
}

// ---------------------------------------------------------------------------
// Helpers

pub(crate) fn encode(s: SlotRef) -> u8 {
    s.p << 4 | s.s
}

fn decode(b: u8) -> SlotRef {
    SlotRef::new((b >> 4) as usize, b & 15)
}

fn occupied(g: &Game, s: SlotRef) -> bool {
    !g.st.slot(s.p as usize, s.s).cards.is_empty()
}

fn atk_base(g: &Game, f: &Frame, target: SlotRef) -> Option<AtkBase> {
    let (p, opp, attack, source) = attack_data(g, f.eff)?;
    Some(AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target })
}

/// The slot types a selector ranges over (for the prompt).
fn sel_types(sel: &SlotSel) -> SVec<u8, 3> {
    let mut v = SVec::new();
    match sel {
        SlotSel::One(_) => v.push(SlotType::Active as u8),
        SlotSel::Bench(_) => v.push(SlotType::Bench as u8),
        SlotSel::Pokemon(_) => {
            v.push(SlotType::Active as u8);
            v.push(SlotType::Bench as u8);
        }
        SlotSel::PokemonBenchFirst(_) => {
            v.push(SlotType::Bench as u8);
            v.push(SlotType::Active as u8);
        }
        SlotSel::Filtered(inner, _) => return sel_types(inner),
    }
    v
}

/// The owner of the Pokémon a selector ranges over.
fn sel_owner(sel: &SlotSel, f: &Frame) -> usize {
    match sel {
        SlotSel::One(SlotExpr::Active(w)) | SlotSel::Bench(w) | SlotSel::Pokemon(w) | SlotSel::PokemonBenchFirst(w) => f.who(*w),
        SlotSel::One(SlotExpr::This) => f.p as usize,
        SlotSel::One(SlotExpr::Picked) => (f.slot >> 4) as usize,
        SlotSel::One(SlotExpr::Attached) => (f.attached_to >> 4) as usize,
        SlotSel::Filtered(inner, _) => sel_owner(inner, f),
    }
}

/// Ask the chooser to pick one of `cands` (resumed at `sub`).
fn ask(g: &mut Game, me: CardId, f: &Frame, pick: &PickSlotSpec, cands: &[SlotRef], sub: u8) {
    let owner = sel_owner(&pick.among, f);
    let chooser = f.who(pick.chooser);
    let player_type = if owner == chooser { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let slots = sel_types(&pick.among);
    let mut blocked: TargetList = SVec::new();
    let pl = &g.st.players[owner];
    if slots.contains(&(SlotType::Active as u8)) && !cands.iter().any(|c| c.s == pl.active) {
        blocked.push(CardTarget::new(player_type, SlotType::Active, 0));
    }
    if slots.contains(&(SlotType::Bench as u8)) {
        for (i, b) in pl.bench.iter().enumerate() {
            if !cands.iter().any(|c| c.s == *b) {
                blocked.push(CardTarget::new(player_type, SlotType::Bench, i as u8));
            }
        }
    }
    let id = g.player_id(chooser);
    g.prompt(
        id,
        pick.msg,
        PromptKind::ChoosePokemon { player_type, slots, min: 1, max: 1, allow_cancel: false, blocked },
        f.cont(me, sub),
    );
}

fn target_pick(t: &SlotTarget) -> Option<&PickSlotSpec> {
    match t {
        SlotTarget::Pick(p) => Some(p),
        SlotTarget::Slot(_) => None,
    }
}

/// The target of the ops that act on one Pokémon.
fn target_of(op: &Op) -> Option<&SlotTarget> {
    match op {
        Op::Heal(h) => Some(&h.target),
        Op::DamageSlot(d) => Some(&d.target),
        Op::PlaceCounters(c) => Some(&c.target),
        _ => None,
    }
}

fn when_of(op: &Op) -> Option<&Cond> {
    match op {
        Op::DamageSlot(d) => Some(&d.when),
        Op::Damage(d) => Some(&d.when),
        Op::Conditions(c) => Some(&c.when),
        Op::KnockOut(k) => Some(&k.when),
        _ => None,
    }
}

fn guard(g: &mut Game, me: CardId, f: &Frame, op: &Op) -> R<bool> {
    match when_of(op) {
        Some(c) => cond_m(g, me, f, c),
        None => Ok(true),
    }
}

/// The Pokémon the player may pick.
fn candidates(g: &mut Game, me: CardId, f: &Frame, pick: &PickSlotSpec) -> R<SVec<SlotRef, 9>> {
    slots_m(g, me, f, &pick.among)
}

// ---------------------------------------------------------------------------
// Entry points

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::RemoveFromPlay(r) => {
            if let Some(slot) = slot_of(g, me, f, r.slot) {
                if !occupied(g, slot) {
                    return Ok(Flow::Next);
                }
                if r.effect_of_attack {
                    if let Some((_, _, attack, _)) = attack_data(g, f.eff) {
                        if attack_effect_prevented_on(g, f.p as usize, slot.p as usize, pack_attack(attack), slot)? {
                            return Ok(Flow::Next);
                        }
                    }
                }
                let dst = zone_ref(f, r.destination);
                move_pokemon_off_board(g, slot, dst, me)?;
            }
            Ok(Flow::Next)
        }
        Op::Damage(d) => {
            if !cond_m(g, me, f, &d.when)? {
                return Ok(Flow::Next);
            }
            let n = num_m(g, me, f, &d.hp)?;
            if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
                match d.op {
                    DamageOp::Add => *damage += n,
                    DamageOp::Set => *damage = n,
                }
            }
            Ok(Flow::Next)
        }
        Op::Heal(_) | Op::DamageSlot(_) | Op::PlaceCounters(_) => {
            let t = target_of(op).expect("op with a target");
            match t {
                SlotTarget::Slot(e) => {
                    if !guard(g, me, f, op)? {
                        return Ok(Flow::Next);
                    }
                    if let Some(s) = slot_of(g, me, f, *e) {
                        let direct = matches!(op, Op::PlaceCounters(PlaceCountersSpec { cause: CounterCause::Direct, .. }));
                        if occupied(g, s) || direct {
                            act(g, me, f, op, s)?;
                        }
                    }
                    Ok(Flow::Next)
                }
                SlotTarget::Pick(pick) => {
                    if let Some(c) = f.recorded_choice(g, me) {
                        if c.answer == CHOICE_NONE || c.len == 0 {
                            return Ok(Flow::Next);
                        }
                        let s = decode(c.items[0]);
                        if occupied(g, s) {
                            act(g, me, f, op, s)?;
                        }
                        return Ok(Flow::Next);
                    }
                    if !guard(g, me, f, op)? {
                        return Ok(Flow::Next);
                    }
                    let cands = candidates(g, me, f, pick)?;
                    if cands.is_empty() {
                        return Ok(Flow::Next);
                    }
                    ask(g, me, f, pick, cands.as_slice(), 1);
                    Ok(Flow::Suspend)
                }
            }
        }
        Op::Conditions(c) => conditions(g, me, f, op, c),
        Op::KnockOut(k) => {
            if !cond_m(g, me, f, &k.when)? {
                return Ok(Flow::Next);
            }
            let Some(slot) = slot_of(g, me, f, k.target) else { return Ok(Flow::Next) };
            if !occupied(g, slot) {
                return Ok(Flow::Next);
            }
            match k.mode {
                KnockOutMode::Direct => {
                    g.st.players[slot.p as usize].slots[slot.s as usize].damage += 999;
                }
                KnockOutMode::Opponent | KnockOutMode::Player => {
                    let Some(b) = atk_base(g, f, slot) else { return Ok(Flow::Next) };
                    if k.mode == KnockOutMode::Opponent {
                        g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
                    } else {
                        g.run_fx(Effect::KnockOutPlayer { b, knocked_out: false, prize_count: 0 })?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::Switch(s) => switch_exec(g, me, f, s),
        Op::Evolve(EvolveSpec { how: EvolveHow::RareCandy }) => {
            let p = f.p as usize;
            let stage2 = stage2_in_hand(g, p);
            let mut blocked: TargetList = SVec::new();
            for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.cdef(c).stage == Stage::Basic as u8 && stage2.iter().any(|s2| matching_stage2(g, c, *s2)) && candy_played_turn(g, p, s)? < g.st.turn {
                    continue;
                }
                blocked.push(t);
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_EVOLVE",
                PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
                f.cont(me, 1),
            );
            Ok(Flow::Suspend)
        }
        Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot, card } }) => {
            let (Some(slot), Some(&card)) = (slot_of(g, me, f, *slot), reg_list(g, f, *card).first()) else { return Ok(Flow::Next) };
            let (p, s) = (slot.p as usize, slot.s);
            let source_card = g.st.slot_pokemon(p, s).unwrap_or(NO_CARD);
            let Some(from) = g.st.locate(card) else { return Ok(Flow::Next) };
            move_cards(g, from, crate::state::ListRef::Slot(p as u8, s), &[card], source_card)?;
            let turn = g.st.turn;
            let sl = &mut g.st.players[p].slots[s as usize];
            crate::engine::game_effect::clear_effects(sl);
            sl.pokemon_played_turn = turn;
            Ok(Flow::Next)
        }
        Op::Devolve(dv) => {
            let Some(slot) = slot_of(g, me, f, dv.slot) else { return Ok(Flow::Next) };
            let (p, s) = (slot.p as usize, slot.s);
            if occupied(g, slot) && g.st.slot_pokemons(p, s).len() > 1 {
                if let Some(b) = atk_base(g, f, slot) {
                    let (_, prevented) = g.run_fx(Effect::Devolve { b })?;
                    if !prevented {
                        devolve_pokemon(g, slot, zone_ref(f, dv.destination))?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::SwapPokemonCard(sw) => {
            // The chosen card goes onto this Pokémon's slot, this card leaves for `into`; it is the
            // same Pokémon (ruling 1840): the state kept on the card moves to the new card.
            let new = reg_list(g, f, sw.cards).first().copied();
            let slot = slot_of(g, me, f, sw.slot);
            if let (Some(new), Some(slot)) = (new, slot) {
                let (p, s) = (slot.p as usize, slot.s);
                let Some(old) = g.st.slot_pokemon(p, s) else { return Ok(Flow::Next) };
                if let Some(src) = g.st.locate(new) {
                    let list = crate::state::ListRef::Slot(slot.p, slot.s);
                    let old_index = g.st.slot(p, s).cards.index_of(old);
                    move_cards(g, src, list, &[new], me)?;
                    let dst = zone_ref(f, sw.into);
                    move_cards(g, list, dst, &[old], me)?;
                    if sw.keep_index {
                        let slot = &mut g.st.players[p].slots[s as usize];
                        if let (Some(ni), Some(oi)) = (slot.cards.index_of(new), old_index) {
                            if ni != oi {
                                slot.cards.remove_at(ni);
                                let at = oi.min(slot.cards.len());
                                slot.cards.insert(at, new);
                            }
                        }
                    }
                    transfer_pokemon_card_state(g, p, old, new);
                }
            }
            Ok(Flow::Next)
        }
        Op::PickSlot(pick) => {
            if let Some(c) = f.recorded_choice(g, me) {
                f.slot = if c.answer == CHOICE_NONE || c.len == 0 { NONE } else { c.items[0] };
                return Ok(Flow::Next);
            }
            let cands = candidates(g, me, f, pick)?;
            // A fixed Pokémon is just selected (nothing to ask).
            if matches!(pick.among, SlotSel::One(_)) {
                f.slot = cands.as_slice().first().map(|s| encode(*s)).unwrap_or(NONE);
                return Ok(Flow::Next);
            }
            if cands.is_empty() {
                f.slot = NONE;
                return Ok(Flow::Next);
            }
            ask(g, me, f, pick, cands.as_slice(), 1);
            Ok(Flow::Suspend)
        }
        Op::SpreadDamage(s) => {
            if let Some(c) = f.recorded_choice(g, me) {
                if c.answer != CHOICE_NONE {
                    spread_damage_carry_out(g, f, s, &c.items[..c.len as usize])?;
                }
                return Ok(Flow::Next);
            }
            Ok(if spread_damage_prompt(g, me, f, s)? { Flow::Suspend } else { Flow::Next })
        }
        Op::SwitchWithActive(w) => {
            if let Some(s) = slot_of(g, me, f, w.target) {
                let p = s.p as usize;
                if g.st.players[p].bench_index_of(s.s).is_some() {
                    crate::engine::turn::switch_pokemon(g, p, s.s)?;
                }
            }
            Ok(Flow::Next)
        }
        Op::Evolve(EvolveSpec { how: EvolveHow::FromDeck { chooser, .. } }) => {
            let p = f.who(*chooser);
            let (_, blocked) = evolve_targets(g, p)?;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            let id = g.player_id(p);
            g.prompt(id, "CHOOSE_POKEMON_TO_EVOLVE", PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked }, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::DamageChosen(d) => {
            if let Some(c) = f.recorded_choice(g, me) {
                if c.answer != CHOICE_NONE {
                    for b in c.items[..c.len as usize].to_vec() {
                        chosen_hit(g, me, f, d, decode(b))?;
                    }
                }
                return Ok(Flow::Next);
            }
            if chosen_ask(g, me, f, d)? {
                Ok(Flow::Suspend)
            } else {
                Ok(Flow::Next)
            }
        }
        Op::SpreadCounters(s) => spread_exec(g, me, f, s),
        Op::MoveCounters(m) => match &m.kind {
            MoveCountersKind::AllFromOne { .. } => move_all_exec(g, me, f, m),
            MoveCountersKind::AnyAmong { who } => move_any_exec(g, me, f, *who),
            MoveCountersKind::MineToOpp { max } => mine_to_opp_exec(g, me, f, *max),
        },
        _ => unimplemented!("spec op not implemented yet (ops/board.rs)"),
    }
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::DamageChosen(d) => {
            for s in first.slots().to_vec() {
                chosen_hit(g, me, f, d, s)?;
            }
            Ok(Flow::Next)
        }
        Op::Evolve(EvolveSpec { how: EvolveHow::FromDeck { chooser, stage, then_stage } }) => evolve_resume(g, me, f, *chooser, *stage, *then_stage, first),
        Op::Heal(_) | Op::DamageSlot(_) | Op::PlaceCounters(_) => {
            if let Some(s) = first.slots().first().copied() {
                if occupied(g, s) {
                    act(g, me, f, op, s)?;
                }
            }
            Ok(Flow::Next)
        }
        Op::Switch(s) => {
            if let Some(slot) = first.slots().first().copied() {
                switch_act(g, me, f, s, slot)?;
            }
            Ok(Flow::Next)
        }
        Op::PickSlot(_) => {
            f.slot = first.slots().first().map(|s| encode(*s)).unwrap_or(NONE);
            Ok(Flow::Next)
        }
        Op::Evolve(EvolveSpec { how: EvolveHow::RareCandy }) => {
            let p = f.p as usize;
            if f.sub == 1 {
                let Some(target) = first.slots().first().copied() else { return Ok(Flow::Next) };
                let Some(base) = g.st.slot_pokemon(target.p as usize, target.s) else { return Ok(Flow::Next) };
                f.slot = encode(target);
                let mut opts = ChooseCardsOpts::new(1, 1, false);
                let hand: Vec<CardId> = g.st.players[p].hand.iter().collect();
                for (i, c) in hand.iter().enumerate() {
                    let d = g.st.cdef(*c);
                    if d.is_pokemon() && d.stage == Stage::Stage2 as u8 && !matching_stage2(g, base, *c) {
                        opts.blocked.push(i as u8);
                    }
                }
                let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Stage2 as u8), ..Filter::none() };
                choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", crate::state::ListRef::Hand(p as u8), filter, opts, f.cont(me, 2));
                return Ok(Flow::Suspend);
            }
            if let Some(c) = first.cards().first().copied() {
                let target = decode(f.slot);
                g.run_fx(Effect::Evolve { p: p as u8, target, card: c })?;
                // It counts as evolving (ruling 1045): the Pokémon loses its Special Conditions and other effects.
                crate::engine::play::finish_evolution(g, p, target)?;
            }
            Ok(Flow::Next)
        }
        Op::SpreadDamage(s) => {
            let items = spread_damage_items(g, f, s, first)?;
            spread_damage_carry_out(g, f, s, &items)?;
            Ok(Flow::Next)
        }
        Op::Conditions(c) => {
            conditions_chosen(g, f, me, c, first)?;
            Ok(Flow::Next)
        }
        Op::SpreadCounters(s) => {
            spread_resume(g, f, s, first)?;
            Ok(Flow::Next)
        }
        Op::MoveCounters(m) => match &m.kind {
            MoveCountersKind::AllFromOne { .. } => move_all_resume(g, me, f, m, first),
            MoveCountersKind::AnyAmong { who } => {
                move_any_resume(g, f, *who, first)?;
                Ok(Flow::Next)
            }
            MoveCountersKind::MineToOpp { max } => {
                mine_to_opp_resume(g, me, f, *max, first)?;
                Ok(Flow::Next)
            }
        },
        _ => Ok(Flow::Next),
    }
}

/// Step D: the choice of an attack effect, asked before the damage.
pub(crate) fn choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::DamageChosen(d) => {
            if chosen_ask(g, me, f, d)? {
                Ok(Flow::Suspend)
            } else {
                f.record(g, me, CHOICE_NONE);
                Ok(Flow::Next)
            }
        }
        Op::Heal(_) | Op::DamageSlot(_) | Op::PlaceCounters(_) => {
            let Some(pick) = target_of(op).and_then(target_pick) else { return Ok(Flow::Next) };
            if !guard(g, me, f, op)? {
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            let cands = candidates(g, me, f, pick)?;
            if cands.is_empty() {
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            ask(g, me, f, pick, cands.as_slice(), 1);
            Ok(Flow::Suspend)
        }
        Op::Switch(s) => {
            let cands = slots_of(g, me, f, &switch_among(s));
            if cands.is_empty() || switch_prevented(g, f, s)? {
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            ask(g, me, f, &switch_pick(s), cands.as_slice(), 1);
            Ok(Flow::Suspend)
        }
        Op::MoveCounters(m) if matches!(m.kind, MoveCountersKind::AllFromOne { .. }) => move_all_exec(g, me, f, m),
        Op::PickSlot(pick) => {
            // A fixed Pokémon is just selected when the effect is carried out.
            if matches!(pick.among, SlotSel::One(_)) {
                return Ok(Flow::Next);
            }
            let cands = candidates(g, me, f, pick)?;
            if cands.is_empty() {
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            ask(g, me, f, pick, cands.as_slice(), 1);
            Ok(Flow::Suspend)
        }
        Op::SpreadDamage(s) => {
            if spread_damage_prompt(g, me, f, s)? {
                return Ok(Flow::Suspend);
            }
            f.record(g, me, CHOICE_NONE);
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::DamageChosen(_) => {
            let items: Vec<u8> = first.slots().iter().map(|s| encode(*s)).collect();
            if items.is_empty() {
                f.record(g, me, CHOICE_NONE);
            } else {
                f.record_items(g, me, CHOICE_YES, &items);
            }
            Ok(Flow::Next)
        }
        Op::MoveCounters(m) => move_all_resume(g, me, f, m, first),
        Op::SpreadDamage(s) => {
            let items = spread_damage_items(g, f, s, first)?;
            f.record_items(g, me, CHOICE_YES, &items);
            Ok(Flow::Next)
        }
        Op::Heal(_) | Op::DamageSlot(_) | Op::PlaceCounters(_) | Op::Switch(_) | Op::PickSlot(_) => {
            match first.slots().first().copied() {
                Some(s) => f.record_items(g, me, CHOICE_YES, &[encode(s)]),
                None => f.record(g, me, CHOICE_NONE),
            }
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn implied_ok(g: &Game, me: CardId, f: &Frame, op: &Op) -> bool {
    match op {
        Op::Heal(h) => match &h.target {
            // A pick needs a Pokémon to choose; checked filters are assumed possible.
            SlotTarget::Pick(p) => !slots_of(g, me, f, &p.among).is_empty(),
            SlotTarget::Slot(_) => true,
        },
        // Used through an attack (Look-Alike Show) a Trainer's switch does nothing when it can't.
        Op::Switch(s) => !s.required || f.via_attack || !slots_of(g, me, f, &switch_among(s)).is_empty(),
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// Acting on a chosen Pokémon

fn act(g: &mut Game, me: CardId, f: &Frame, op: &Op, slot: SlotRef) -> R {
    match op {
        Op::Heal(h) => {
            let n = num_m(g, me, f, &h.hp)?;
            match h.via {
                HealVia::Attack => {
                    if let Some(b) = atk_base(g, f, slot) {
                        g.run_fx(Effect::HealTarget { b, damage: n })?;
                    }
                }
                HealVia::Effect => {
                    g.run_fx(Effect::Heal { p: f.p, target: slot, damage: n })?;
                }
            }
            if h.clear_conditions {
                g.st.players[slot.p as usize].slots[slot.s as usize].special_conditions.clear();
            }
        }
        Op::DamageSlot(d) => {
            let mut n = num_m(g, me, f, &d.hp)?;
            if d.target_damage_mul != 0 {
                n += d.target_damage_mul * g.st.slot(slot.p as usize, slot.s).damage;
            }
            damage_by(g, f, d.calc, n, slot)?;
        }
        Op::PlaceCounters(c) => {
            let n = num_m(g, me, f, &c.counters)? * 10;
            match c.cause {
                CounterCause::Effect => {
                    g.run_fx(Effect::PlaceDamageCounters { p: f.p, target: slot, damage: n, source: me })?;
                }
                CounterCause::Direct => {
                    g.st.players[slot.p as usize].slots[slot.s as usize].damage += n;
                }
                CounterCause::Attack => {
                    if let Some(b) = atk_base(g, f, slot) {
                        g.run_fx(Effect::PutCounters { b, damage: n })?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Special Conditions

const CONDITION_NAMES: &[&str] = &["Paralyzed", "Confused", "Asleep", "Poisoned", "Burned"];

fn conditions(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, c: &ConditionsSpec) -> R<Flow> {
    if !guard(g, me, f, op)? {
        return Ok(Flow::Next);
    }
    let Some(slot) = slot_of(g, me, f, c.target) else { return Ok(Flow::Next) };
    if !occupied(g, slot) {
        return Ok(Flow::Next);
    }
    let (p, s) = (slot.p as usize, slot.s);
    match c.change {
        ConditionChange::Add(cs) => {
            if c.gate == Gate::TrainerTarget {
                let (t, prevented) = g.run_fx(Effect::TrainerTarget { p: f.p, card: me, target: Some(slot) })?;
                if prevented || matches!(t, Effect::TrainerTarget { target: None, .. }) {
                    return Ok(Flow::Next);
                }
            }
            match c.cause {
                Cause::Attack => {
                    let Some(b) = atk_base(g, f, slot) else { return Ok(Flow::Next) };
                    let mut v = SVec::new();
                    for x in cs {
                        v.push(*x as u8);
                    }
                    g.run_fx(Effect::AddSpecialConditions { b, conditions: v, poison_damage: None, burn_damage: None, confusion_damage: None })?;
                }
                Cause::Ability => add_special_conditions_to_player_active(g, p, me, cs)?,
                Cause::Direct => {
                    for x in cs {
                        crate::engine::phase::add_condition(&mut g.st.players[p].slots[s as usize], *x);
                    }
                }
            }
            Ok(Flow::Next)
        }
        ConditionChange::RemoveAll => {
            g.st.players[p].slots[s as usize].special_conditions.clear();
            Ok(Flow::Next)
        }
        ConditionChange::RemoveChosen => {
            let conds: Vec<u8> = g.st.slot(p, s).special_conditions.as_slice().to_vec();
            if conds.len() == 1 {
                crate::engine::phase::remove_condition(g, p, s, SpecialCondition::from_u8(conds[0]));
            } else if conds.len() > 1 {
                let mut disabled = 0u16;
                for i in 0..CONDITION_NAMES.len() {
                    if !conds.contains(&(i as u8)) {
                        disabled |= 1 << i;
                    }
                }
                let id = g.player_id(f.p as usize);
                g.prompt(
                    id,
                    "CHOOSE_OPTION",
                    PromptKind::SelectOption { values: CONDITION_NAMES, allow_cancel: false, default_value: conds[0] as i32, disabled: Some(disabled) },
                    f.cont(me, 1),
                );
                return Ok(Flow::Suspend);
            }
            Ok(Flow::Next)
        }
    }
}

fn conditions_chosen(g: &mut Game, f: &Frame, me: CardId, c: &ConditionsSpec, first: Res) -> R {
    let Some(slot) = slot_of(g, me, f, c.target) else { return Ok(()) };
    let (p, s) = (slot.p as usize, slot.s);
    let choice = match first {
        Res::Int(i) => i,
        // A missing answer stands in with the default (the first listed).
        _ => match g.st.slot(p, s).special_conditions.as_slice().first() {
            Some(x) => *x as i32,
            None => return Ok(()),
        },
    };
    crate::engine::phase::remove_condition(g, p, s, SpecialCondition::from_u8(choice as u8));
    Ok(())
}

// ---------------------------------------------------------------------------
// Switching

/// The Pokémon a switch can bring to the Active Spot.
fn switch_among(s: &SwitchSpec) -> SlotSel {
    match (s.kind, s.side) {
        // Blocked: a Pokémon that is not Basic (a slot without a Pokémon card, a Fossil, is not).
        (SwitchKind::PlainBasic, Who::Me) => SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Not(&SlotPred::Top(Pred::Not(&Pred::Basic)))),
        (SwitchKind::PlainBasic, Who::Opp) => SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Not(&SlotPred::Top(Pred::Not(&Pred::Basic)))),
        _ => SlotSel::Bench(s.side),
    }
}

fn switch_pick(s: &SwitchSpec) -> PickSlotSpec {
    PickSlotSpec { chooser: s.chooser, among: switch_among(s), msg: s.msg }
}

/// A fresh attack effect for the attack in use (Gust and switch-out effects
/// are built on a new AttackEffect).
fn fresh_attack(g: &mut Game, f: &Frame) -> Option<(crate::effects::EffId, AtkBase)> {
    let (p, opp, attack, _) = attack_data(g, f.eff)?;
    let source = SlotRef::new(p as usize, g.st.players[p as usize].active);
    let atk = g.new_fx(Effect::Attack {
        p,
        opp,
        attack,
        damage: 0,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source,
        barrage_used: false,
    });
    let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
    Some((atk, AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }))
}

/// Is the switch-out of the opponent's Active prevented (probe)?
fn switch_prevented(g: &mut Game, f: &Frame, s: &SwitchSpec) -> R<bool> {
    if s.kind != SwitchKind::SwitchOut {
        return Ok(false);
    }
    let Some((atk, b)) = fresh_attack(g, f) else { return Ok(false) };
    let r = g.run_fx(Effect::SwitchOutOpponentsActive { b, bench_target: None });
    g.release_fx(atk);
    Ok(r?.1)
}

fn switch_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SwitchSpec) -> R<Flow> {
    if s.kind == SwitchKind::Picked {
        if let Some(slot) = slot_of(g, me, f, SlotExpr::Picked) {
            if occupied(g, slot) {
                switch_act(g, me, f, s, slot)?;
            }
        }
        return Ok(Flow::Next);
    }
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer == CHOICE_NONE || c.len == 0 {
            return Ok(Flow::Next);
        }
        let slot = decode(c.items[0]);
        if occupied(g, slot) {
            switch_act(g, me, f, s, slot)?;
        }
        return Ok(Flow::Next);
    }
    let cands = slots_of(g, me, f, &switch_among(s));
    if cands.is_empty() || switch_prevented(g, f, s)? {
        return Ok(Flow::Next);
    }
    ask(g, me, f, &switch_pick(s), cands.as_slice(), 1);
    Ok(Flow::Suspend)
}

fn switch_act(g: &mut Game, me: CardId, f: &mut Frame, s: &SwitchSpec, slot: SlotRef) -> R {
    let side = f.who(s.side);
    // The Pokémon that leaves the Active Spot is the picked slot afterwards (for effects on it).
    f.slot = encode(SlotRef::new(side, g.st.players[side].active));
    // The switch only acts on the side's own Bench.
    if slot.p as usize != side {
        return Ok(());
    }
    match s.kind {
        SwitchKind::Plain | SwitchKind::PlainBasic => crate::engine::turn::switch_pokemon(g, side, slot.s),
        SwitchKind::SilentAbilityEffect => {
            let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: f.p, power: crate::effects::PowerRef { card: me, index: 0 }, card: me, target: Some(slot) })?;
            if let Effect::EffectOfAbility { target: Some(_), .. } = fx {
                let a = g.st.players[side].active;
                crate::engine::game_effect::clear_effects(&mut g.st.players[side].slots[a as usize]);
                crate::engine::turn::switch_pokemon_silent(g, side, slot.s)?;
            }
            Ok(())
        }
        SwitchKind::Silent | SwitchKind::Picked => {
            let a = g.st.players[side].active;
            crate::engine::game_effect::clear_effects(&mut g.st.players[side].slots[a as usize]);
            crate::engine::turn::switch_pokemon_silent(g, side, slot.s)
        }
        SwitchKind::Gust => {
            let Some((atk, mut b)) = fresh_attack(g, f) else { return Ok(()) };
            b.target = slot;
            let r = g.run_fx(Effect::GustOpponentBench { b });
            g.release_fx(atk);
            r.map(|_| ())
        }
        SwitchKind::SwitchOut => {
            let Some((atk, b)) = fresh_attack(g, f) else { return Ok(()) };
            let r = g.run_fx(Effect::SwitchOutOpponentsActive { b, bench_target: Some(slot) });
            g.release_fx(atk);
            r.map(|_| ())
        }
    }
}

// ---------------------------------------------------------------------------
// Spreading and moving counters

fn spread_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SpreadCountersSpec) -> R<Flow> {
    let p = f.who(s.chooser);
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (sl, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let hp = crate::engine::check::check_hp(g, p, sl)?;
        max_allowed.push((t, hp + s.cap_bonus_hp));
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::PutDamage {
            player_type: PlayerType::BottomPlayer,
            slots,
            damage: s.total_hp,
            max_allowed,
            allow_cancel: false,
            blocked: SVec::new(),
            allow_partial: true,
            damage_multiple: 10,
        },
        f.cont(me, 1),
    );
    Ok(Flow::Suspend)
}

fn spread_resume(g: &mut Game, f: &Frame, s: &SpreadCountersSpec, first: Res) -> R {
    let p = f.who(s.chooser);
    let Some((pl, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    let map: SVec<(CardTarget, i32), 16> = match first {
        Res::DamageMap(m) => m,
        _ => SVec::new(),
    };
    if let Effect::Attack { damage: d, .. } = g.e_mut(f.eff) {
        *d = 0;
    }
    for (t, damage) in map.iter() {
        let target = get_target(&g.st, p, *t)?;
        let b = AtkBase { attack_effect: f.eff, player: pl, opponent: opp, attack, source, target };
        g.run_fx(Effect::PutCounters { b, damage: *damage })?;
        if let Effect::Attack { damage: d, .. } = g.e_mut(f.eff) {
            *d = *damage * s.damage_per_hp;
        }
    }
    Ok(())
}

/// Any number of counters move between the Pokémon of one side (a MoveDamage prompt).
fn move_any_exec(g: &mut Game, me: CardId, f: &mut Frame, who: Who) -> R<Flow> {
    let owner = f.who(who);
    let player_type = if owner == f.p as usize { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (sl, _, t) in for_each_pokemon(g, owner, player_type).iter().copied() {
        let hp = crate::engine::check::check_hp(g, owner, sl)?;
        max_allowed.push((t, hp));
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(f.p as usize);
    g.prompt(
        id,
        "MOVE_DAMAGE",
        PromptKind::MoveDamage { player_type, slots, max_allowed, o: MoveOpts::default(), single_source: false, single_destination: false, damage_multiple: 10 },
        f.cont(me, 1),
    );
    Ok(Flow::Suspend)
}

fn move_any_resume(g: &mut Game, f: &Frame, _who: Who, first: Res) -> R {
    let p = f.p as usize;
    let Res::DamageTransfers(transfers) = first else { return Ok(()) };
    let Some((_, opp, attack, asource)) = attack_data(g, f.eff) else { return Ok(()) };
    for (from, to) in damage_transfers(transfers.as_slice()) {
        let source = get_target(&g.st, p, from)?;
        let target = get_target(&g.st, p, to)?;
        if g.st.slot(source.p as usize, source.s).damage >= 10 {
            let b = AtkBase { attack_effect: f.eff, player: p as u8, opponent: opp, attack, source: asource, target: source };
            let (_, from_prevented) = g.run_fx(Effect::PutCounters { b, damage: 0 })?;
            if from_prevented {
                continue;
            }
            g.st.players[source.p as usize].slots[source.s as usize].damage -= 10;
            let b = AtkBase { attack_effect: f.eff, player: p as u8, opponent: opp, attack, source: asource, target };
            let (_, to_prevented) = g.run_fx(Effect::PutCounters { b, damage: 0 })?;
            if !to_prevented {
                g.st.players[target.p as usize].slots[target.s as usize].damage += 10;
            }
        }
    }
    Ok(())
}

/// All the counters of one Pokémon to another: asks the source, then the
/// destination (the source rides in the resume point), or reads the step-D answer.
fn move_all_exec(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveCountersSpec) -> R<Flow> {
    let MoveCountersKind::AllFromOne { from, .. } = &m.kind else { return Ok(Flow::Next) };
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer != CHOICE_NONE && c.len >= 2 {
            move_all_act(g, f, decode(c.items[0]), decode(c.items[1]))?;
        }
        return Ok(Flow::Next);
    }
    let cands = candidates(g, me, f, from)?;
    if cands.is_empty() {
        if f.phase == Phase::Choices {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    ask(g, me, f, from, cands.as_slice(), 1);
    Ok(Flow::Suspend)
}

fn move_all_resume(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveCountersSpec, first: Res) -> R<Flow> {
    let MoveCountersKind::AllFromOne { to, .. } = &m.kind else { return Ok(Flow::Next) };
    let Some(picked) = first.slots().first().copied() else { return Ok(Flow::Next) };
    if f.sub < 0x80 {
        let cands = candidates(g, me, f, to)?;
        ask(g, me, f, to, cands.as_slice(), 0x80 | encode(picked));
        return Ok(Flow::Suspend);
    }
    let src = decode(f.sub & 0x7F);
    if f.phase == Phase::Choices {
        f.record_items(g, me, CHOICE_YES, &[encode(src), encode(picked)]);
    } else {
        move_all_act(g, f, src, picked)?;
    }
    Ok(Flow::Next)
}

fn move_all_act(g: &mut Game, f: &Frame, src: SlotRef, tgt: SlotRef) -> R {
    let p = f.p as usize;
    let move_damage = g.st.slot(src.p as usize, src.s).damage;
    if move_damage <= 0 {
        return Ok(());
    }
    let (_, prevented) = g.run_fx(Effect::MoveDamageCounters { p: p as u8 })?;
    if prevented {
        return Ok(());
    }
    let Some((_, opp, attack, _)) = attack_data(g, f.eff) else { return Ok(()) };
    let b = AtkBase { attack_effect: f.eff, player: p as u8, opponent: opp, attack, source: src, target: tgt };
    let (fin, prevented) = g.run_fx(Effect::MoveCounters { b, damage: move_damage })?;
    if let Effect::MoveCounters { b, damage } = fin {
        let s = &mut g.st.players[b.source.p as usize].slots[b.source.s as usize];
        s.damage -= damage;
        if s.damage < 0 {
            s.damage = 0;
        }
        if !prevented {
            g.st.players[b.target.p as usize].slots[b.target.s as usize].damage += damage;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Short forms for spec files

pub const OPP_ACTIVE: SlotExpr = SlotExpr::Active(Who::Opp);
pub const MY_ACTIVE: SlotExpr = SlotExpr::Active(Who::Me);

/// The opponent's Active Pokémon is now affected by these Special Conditions.
pub const fn inflict(cs: &'static [SpecialCondition], cause: Cause) -> Op {
    Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(cs), cause, gate: Gate::None, when: Cond::True })
}

/// "This Pokémon also does N damage to itself."
pub const fn self_damage(hp: i32) -> Op {
    Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(hp), target_damage_mul: 0, calc: DamageCalc::Deal, when: Cond::True })
}

/// "Switch this Pokémon with 1 of your Benched Pokémon."
pub const fn switch_self() -> Op {
    Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false })
}

/// Heal damage from this Pokémon.
pub const fn heal_active(hp: i32, via: HealVia) -> Op {
    Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(hp), via, clear_conditions: false })
}

/// "N more damage" when the condition holds.
pub const fn more_damage_if(hp: i32, when: Cond) -> Op {
    Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(hp), when })
}

/// The attack's damage is this number (N times, or set to a value).
pub const fn damage_is(hp: Num) -> Op {
    Op::Damage(DamageSpec { op: DamageOp::Set, hp, when: Cond::True })
}

// ---------------------------------------------------------------------------
// MoveCounters::MineToOpp (S3 agent 3)

fn mine_to_opp_exec(g: &mut Game, me: CardId, f: &mut Frame, max: u8) -> R<Flow> {
    let p = f.p as usize;
    let o = 1 - p;
    let mine = for_each_pokemon(g, p, PlayerType::BottomPlayer);
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (s, _, t) in mine.iter().copied() {
        let hp = crate::engine::check::check_hp(g, p, s)?;
        max_allowed.push((t, hp));
    }
    let mut opts = MoveOpts { allow_cancel: false, min: 1, max: Some(max), ..Default::default() };
    for (_, _, t) in mine.iter().copied() {
        opts.blocked_to.push(t);
    }
    for (_, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        opts.blocked_from.push(t);
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    g.prompt(id, "MOVE_DAMAGE", PromptKind::RemoveDamage { player_type: PlayerType::Any, slots, max_allowed, o: opts, same_target: true }, f.cont(me, 1));
    Ok(Flow::Suspend)
}

fn mine_to_opp_resume(g: &mut Game, me: CardId, f: &Frame, max: u8, first: Res) -> R {
    let p = f.p as usize;
    let Res::DamageTransfers(transfers) = first else { return Ok(()) };
    let limit = max as i32 * 10;
    let mut total = 0;
    for (from, to) in damage_transfers(transfers.as_slice()) {
        let source = get_target(&g.st, p, from)?;
        let target = get_target(&g.st, p, to)?;
        let src_damage = g.st.slot(source.p as usize, source.s).damage;
        let damage_to_move = (limit - total).min(10.min(src_damage));
        if damage_to_move > 0 {
            let (_, prevented) = g.run_fx(Effect::MoveDamageCounters { p: p as u8 })?;
            if prevented {
                continue;
            }
            g.st.players[source.p as usize].slots[source.s as usize].damage -= damage_to_move;
            g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target, damage: damage_to_move, source: me })?;
            total += damage_to_move;
        }
        if total >= limit {
            break;
        }
    }
    Ok(())
}

// S3 appends: spreading damage

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpreadApply {
    /// PutCountersEffect on each target (an effect of the attack).
    Counters,
    /// Damage to each target (Deal for the Active, Put for the Bench), no Weakness for the Bench.
    Damage,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpreadSlots {
    Bench,
    Pokemon,
}

/// "Put N damage counters / do N damage ... to your opponent's Pokémon in any way you like":
/// one allocation prompt (no cancel, no partial answer), asked at step D for an attack and
/// carried out after the damage.
pub struct SpreadDamageSpec {
    pub chooser: Who,
    /// Whose Pokémon receive it.
    pub side: Who,
    pub slots: SpreadSlots,
    pub total_hp: i32,
    /// The allocation moves in steps of this many HP.
    pub unit_hp: i32,
    /// Each Pokémon takes at most its printed HP plus this much (None: no cap).
    pub cap_bonus_hp: Option<i32>,
    pub apply: SpreadApply,
}

/// Ask for the allocation; false when there is nobody to put it on.
fn spread_damage_prompt(g: &mut Game, me: CardId, f: &Frame, s: &SpreadDamageSpec) -> R<bool> {
    let chooser = f.who(s.chooser);
    let side = f.who(s.side);
    let pl = &g.st.players[side];
    let has_target = match s.slots {
        SpreadSlots::Bench => pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()),
        SpreadSlots::Pokemon => !pl.in_play().is_empty(),
    };
    if !has_target {
        return Ok(false);
    }
    let player_type = if side == chooser { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (_, c, t) in for_each_pokemon(g, side, player_type).iter().copied() {
        let cap = match s.cap_bonus_hp {
            Some(b) => g.st.cdef(c).hp + b,
            None => 9999,
        };
        max_allowed.push((t, cap));
    }
    let mut slots = SVec::new();
    if s.slots == SpreadSlots::Pokemon {
        slots.push(SlotType::Active as u8);
    }
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(chooser);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::PutDamage { player_type, slots, damage: s.total_hp, max_allowed, allow_cancel: false, blocked: SVec::new(), allow_partial: false, damage_multiple: s.unit_hp },
        f.cont(me, 1),
    );
    Ok(true)
}

/// The answer as (slot, units of `unit_hp`) bytes.
fn spread_damage_items(g: &Game, f: &Frame, s: &SpreadDamageSpec, first: Res) -> R<Vec<u8>> {
    let chooser = f.who(s.chooser);
    let map: SVec<(CardTarget, i32), 16> = match first {
        Res::DamageMap(m) => m,
        _ => SVec::new(),
    };
    let mut out = Vec::new();
    for (t, damage) in map.iter() {
        let slot = get_target(&g.st, chooser, *t)?;
        out.push(encode(slot));
        out.push((*damage / s.unit_hp.max(1)).clamp(0, 255) as u8);
    }
    Ok(out)
}

fn spread_damage_carry_out(g: &mut Game, f: &Frame, s: &SpreadDamageSpec, items: &[u8]) -> R {
    for pair in items.chunks(2).filter(|c| c.len() == 2) {
        let slot = decode(pair[0]);
        let damage = pair[1] as i32 * s.unit_hp;
        match s.apply {
            SpreadApply::Damage => deal_or_put_damage(g, f.eff, damage, slot)?,
            SpreadApply::Counters => {
                if let Some(b) = atk_base(g, f, slot) {
                    g.run_fx(Effect::PutCounters { b, damage })?;
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rare Candy (S3 agent 3)

/// `isMatchingStage2(stage1, basic, stage2)`.
pub fn matching_stage2(g: &Game, basic: CardId, stage2: CardId) -> bool {
    let b = g.st.cdef(basic).name;
    let s2 = g.st.cdef(stage2).evolves_from;
    crate::gen::stage1::ALL_STAGE1.iter().any(|(n, from)| *n == s2 && *from == b)
}

pub fn stage2_in_hand(g: &Game, p: usize) -> Vec<CardId> {
    g.st.players[p].hand.iter().filter(|c| {
        let d = g.st.cdef(*c);
        d.is_pokemon() && d.stage == Stage::Stage2 as u8
    }).collect()
}

/// Ask the attacker to pick the Pokémon (resumed at 1); false when there is nobody to pick.
fn chosen_ask(g: &mut Game, me: CardId, f: &Frame, d: &DamageChosenSpec) -> R<bool> {
    let cands = slots_m(g, me, f, &d.among)?;
    let k = (d.count.max(0) as usize).min(cands.len());
    if k == 0 {
        return Ok(false);
    }
    let owner = sel_owner(&d.among, f);
    let player_type = if owner == f.p as usize { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let slots = sel_types(&d.among);
    let mut blocked: TargetList = SVec::new();
    let pl = &g.st.players[owner];
    if slots.contains(&(SlotType::Active as u8)) && !cands.iter().any(|c| c.s == pl.active) {
        blocked.push(CardTarget::new(player_type, SlotType::Active, 0));
    }
    if slots.contains(&(SlotType::Bench as u8)) {
        for (i, b) in pl.bench.iter().enumerate() {
            if !cands.iter().any(|c| c.s == *b) {
                blocked.push(CardTarget::new(player_type, SlotType::Bench, i as u8));
            }
        }
    }
    let id = g.player_id(f.p as usize);
    g.prompt(id, d.msg, PromptKind::ChoosePokemon { player_type, slots, min: k as u8, max: k as u8, allow_cancel: false, blocked }, f.cont(me, 1));
    Ok(true)
}

fn chosen_hit(g: &mut Game, me: CardId, f: &Frame, d: &DamageChosenSpec, slot: SlotRef) -> R {
    if !occupied(g, slot) {
        return Ok(());
    }
    let n = num_m(g, me, f, &d.hp)?;
    damage_by(g, f, d.calc, n, slot)
}

/// `n` damage to the Pokémon the way `calc` says.
fn damage_by(g: &mut Game, f: &Frame, calc: DamageCalc, n: i32, slot: SlotRef) -> R {
    match calc {
        DamageCalc::Auto => deal_or_put_damage(g, f.eff, n, slot)?,
        DamageCalc::Put => put_damage(g, f.eff, n, slot)?,
        DamageCalc::Direct => g.st.players[slot.p as usize].slots[slot.s as usize].damage += n,
        DamageCalc::Deal => {
            if let Some(b) = atk_base(g, f, slot) {
                g.run_fx(Effect::DealDamage { b, damage: n })?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Evolve (Grand Tree)

/// `CheckPokemonPlayedTurnEffect`: (pokemonPlayedTurn, canEvolveOnFirstTurn).
fn played_turn(g: &mut Game, p: usize, s: SlotId) -> R<(i32, bool)> {
    let target = SlotRef::new(p, s);
    let played = g.st.slot(p, s).pokemon_played_turn;
    let (e, _) = g.run_fx(Effect::CheckPokemonPlayedTurn { p: p as u8, target, pokemon_played_turn: played, can_evolve_on_first_turn: false })?;
    Ok(match e {
        Effect::CheckPokemonPlayedTurn { pokemon_played_turn, can_evolve_on_first_turn, .. } => (pokemon_played_turn, can_evolve_on_first_turn),
        _ => (played, false),
    })
}

/// A Pokémon can't be evolved during its owner's first turn (the PlayPokemonEffect test), unless it has its own exception.
fn first_turn_blocked(g: &Game, p: usize, can_evolve_on_first_turn: bool) -> bool {
    g.st.turn <= 2 && !g.st.players[p].can_evolve && !can_evolve_on_first_turn
}

fn evolves_from_any(name: &str) -> bool {
    crate::gen::evolutions::ALL_EVOLUTIONS.iter().any(|(_, from)| *from == name)
}

/// Whether some Basic Pokémon of `p` can evolve now (not put into play this turn, not in the first turn)
/// into a card the game knows, and the Pokémon prompt's blocked targets.
pub(crate) fn evolve_targets(g: &mut Game, p: usize) -> R<(bool, TargetList)> {
    let turn = g.st.turn as i32;
    let mut any = false;
    let mut blocked: TargetList = SVec::new();
    for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Basic as u8 {
            blocked.push(t);
            continue;
        }
        let (played, first_ok) = played_turn(g, p, s)?;
        if played == turn || first_turn_blocked(g, p, first_ok) {
            blocked.push(t);
            continue;
        }
        if evolves_from_any(g.st.cdef(c).name) {
            any = true;
        }
    }
    Ok((any, blocked))
}

/// Deck prompt for an evolution of `from` (`stage`), blocking deck Pokémon that evolve from something else.
fn evolution_prompt(g: &mut Game, p: usize, from: &'static str, stage: Stage, cont: crate::game::Cont) {
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.evolves_from != from {
            blocked.push(i as u8);
        }
    }
    let mut opts = ChooseCardsOpts::new(1, 1, true);
    opts.blocked = blocked;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(stage as u8), evolves_from: Some(from), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), filter, opts, cont);
}

fn evolve_with(g: &mut Game, me: CardId, p: usize, t: SlotRef, card: CardId) -> R {
    move_cards(g, ListRef::Deck(p as u8), t.list(), &[card], me)?;
    let turn = g.st.turn;
    let slot = &mut g.st.players[t.p as usize].slots[t.s as usize];
    crate::engine::game_effect::clear_effects(slot);
    slot.pokemon_played_turn = turn;
    Ok(())
}

fn evolve_resume(g: &mut Game, me: CardId, f: &mut Frame, chooser: Who, stage: Stage, then_stage: Option<Stage>, first: Res) -> R<Flow> {
    let p = f.who(chooser);
    match f.sub & 0xF0 {
        0x00 => {
            // The Pokémon was chosen.
            let Some(t) = first.slots().first().copied() else { return Ok(Flow::Next) };
            let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) else { return Ok(Flow::Next) };
            let (played, first_ok) = played_turn(g, p, t.s)?;
            if g.st.cdef(c).stage != Stage::Basic as u8 || played == g.st.turn as i32 || first_turn_blocked(g, p, first_ok) {
                return Ok(Flow::Next);
            }
            let name = g.st.cdef(c).name;
            evolution_prompt(g, p, name, stage, f.cont(me, 0x20 | t.s));
            Ok(Flow::Suspend)
        }
        0x20 => {
            let t = SlotRef::new(p, f.sub & 0x0F);
            let Some(evo) = first.cards().first().copied() else { return Ok(Flow::Next) };
            evolve_with(g, me, p, t, evo)?;
            let name = g.st.cdef(evo).name;
            if let Some(st2) = then_stage {
                if evolves_from_any(name) {
                    evolution_prompt(g, p, name, st2, f.cont(me, 0x30 | t.s));
                    return Ok(Flow::Suspend);
                }
            }
            Ok(Flow::Next)
        }
        _ => {
            let t = SlotRef::new(p, f.sub & 0x0F);
            if let Some(c) = first.cards().first().copied() {
                evolve_with(g, me, p, t, c)?;
            }
            Ok(Flow::Next)
        }
    }
}

/// `CheckPokemonPlayedTurnEffect` pokemonPlayedTurn, as Rare Candy reads it.
fn candy_played_turn(g: &mut Game, p: usize, s: crate::state::SlotId) -> R<i32> {
    Ok(played_turn(g, p, s)?.0)
}

/// `canUseRareCandy`.
pub fn rare_candy_usable(g: &mut Game, p: usize) -> R<bool> {
    // A player's first turn is turn 1 or 2 (R7F-14, ruling 689).
    if g.st.turn == 1 || g.st.turn == 2 {
        return Ok(false);
    }
    let stage2 = stage2_in_hand(g, p);
    // Evolution Jammer (Bronzong TEF): the player can't evolve.
    if stage2.is_empty() || g.st.players[p].cannot_evolve_pokemon_cards {
        return Ok(false);
    }
    let mut ok = false;
    for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Basic as u8 || !stage2.iter().any(|s2| matching_stage2(g, c, *s2)) {
            continue;
        }
        if candy_played_turn(g, p, s)? < g.st.turn {
            ok = true;
        }
    }
    Ok(ok)
}

/// `DEVOLVE_POKEMON(store, state, target, destination)`.
pub fn devolve_pokemon(g: &mut Game, t: SlotRef, dest: crate::state::ListRef) -> R {
    let (tp, ts) = (t.p as usize, t.s);
    let pokemons = g.st.slot_pokemons(tp, ts);
    let top = g.st.slot_pokemon(tp, ts);
    let top_def = top.map(|c| g.st.cdef(c));
    if let (Some(_), Some(d)) = (top, top_def) {
        if d.has_tag(tag::POKEMON_LV_X) {
            if pokemons.len() == 2 && pokemons.iter().any(|c| g.st.cdef(*c).stage == Stage::Basic as u8) {
                return Ok(());
            }
            let cards: Vec<CardId> = pokemons.iter().copied().filter(|c| g.st.cdef(*c).name == d.name).collect();
            crate::prefabs::move_cards(g, t.list(), dest, &cards, NO_CARD)?;
            let turn = g.st.turn;
            let slot = &mut g.st.players[tp].slots[ts as usize];
            crate::engine::game_effect::clear_effects(slot);
            slot.pokemon_played_turn = turn;
            return Ok(());
        }
    }
    // CardTag.LEGEND is TAG_NAMES index 30.
    let special = top_def.map(|d| d.has_tag(tag::POKEMON_VUNION) || d.has_tag(30)).unwrap_or(false);
    if pokemons.len() > 1 && !special {
        if let Some(top) = top {
            // MOVE_CARD_TO: findCardList(card).moveCardTo(card, destination).
            if let Some(src) = g.st.locate(top) {
                g.move_card_to(src, top, dest);
            }
        }
        let turn = g.st.turn;
        let slot = &mut g.st.players[tp].slots[ts as usize];
        crate::engine::game_effect::clear_effects(slot);
        slot.pokemon_played_turn = turn;
    }
    Ok(())
}
