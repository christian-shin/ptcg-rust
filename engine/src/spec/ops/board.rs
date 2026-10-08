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

use super::super::run::{Flow, Frame, Phase, CHOICE_NONE, CHOICE_YES};
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
    /// Each of the selected Pokémon, in order (S3).
    Each(SlotSel),
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
}

pub struct MoveCountersSpec {
    pub kind: MoveCountersKind,
}
/// A Pokémon evolves with a card from the deck (a ChoosePokemon prompt over the Basic Pokémon that can evolve
/// now, then a ChooseCards prompt over the deck for a `stage` card that evolves from it, cancellable); with
/// `then_stage` a second card evolving from the first is offered. The deck is not shuffled here.
pub struct EvolveSpec {
    pub chooser: Who,
    pub stage: Stage,
    pub then_stage: Option<Stage>,
}

/// This Pokémon (the slot `target`) switches with the Active Pokémon when it is on the Bench.
pub struct SwitchWithActiveSpec {
    pub target: SlotExpr,
}
pub struct DevolveSpec {}
pub struct SwapPokemonCardSpec {}
/// Put a Pokémon and all cards attached to it into a zone.
pub struct RemoveFromPlaySpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
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

/// The attacker picks one of the Pokémon in `among` (asked at step D); after the damage it goes with
/// all its attached cards into its owner's deck, unless effects of attacks on it are prevented.
pub struct RemovePickedSpec {
    pub among: SlotSel,
    pub msg: &'static str,
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

fn encode(s: SlotRef) -> u8 {
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
        SlotSel::One(SlotExpr::Attacker) => (f.ctx >> 4 & 1) as usize,
        SlotSel::One(SlotExpr::Attached) => (f.ctx >> 4 & 1) as usize,
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
        SlotTarget::Slot(_) | SlotTarget::Each(_) => None,
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
                        if occupied(g, s) {
                            act(g, me, f, op, s)?;
                        }
                    }
                    Ok(Flow::Next)
                }
                SlotTarget::Each(sel) => {
                    if !guard(g, me, f, op)? {
                        return Ok(Flow::Next);
                    }
                    for s in slots_m(g, me, f, sel)?.iter() {
                        if occupied(g, *s) {
                            act(g, me, f, op, *s)?;
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
        Op::SwitchWithActive(w) => {
            if let Some(s) = slot_of(g, me, f, w.target) {
                let p = s.p as usize;
                if g.st.players[p].bench_index_of(s.s).is_some() {
                    crate::engine::turn::switch_pokemon(g, p, s.s)?;
                }
            }
            Ok(Flow::Next)
        }
        Op::Evolve(e) => {
            let p = f.who(e.chooser);
            let (_, blocked) = evolve_targets(g, p)?;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            let id = g.player_id(p);
            g.prompt(id, "CHOOSE_POKEMON_TO_EVOLVE", PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked }, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::RemovePicked(r) => {
            let d = DamageChosenSpec { among: r.among_clone(), count: 1, hp: Num::Lit(0), calc: DamageCalc::Auto, msg: r.msg };
            if let Some(c) = f.recorded_choice(g, me) {
                if c.answer != CHOICE_NONE && c.len > 0 {
                    remove_picked(g, me, f, decode(c.items[0]))?;
                }
                return Ok(Flow::Next);
            }
            if chosen_ask(g, me, f, &d)? {
                Ok(Flow::Suspend)
            } else {
                Ok(Flow::Next)
            }
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
        Op::Evolve(e) => evolve_resume(g, me, f, e, first),
        Op::RemovePicked(_) => {
            if let Some(s) = first.slots().first().copied() {
                remove_picked(g, me, f, s)?;
            }
            Ok(Flow::Next)
        }
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
        Op::RemovePicked(r) => {
            let d = DamageChosenSpec { among: r.among_clone(), count: 1, hp: Num::Lit(0), calc: DamageCalc::Auto, msg: r.msg };
            if chosen_ask(g, me, f, &d)? {
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
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::DamageChosen(_) | Op::RemovePicked(_) => {
            let items: Vec<u8> = first.slots().iter().map(|s| encode(*s)).collect();
            if items.is_empty() {
                f.record(g, me, CHOICE_NONE);
            } else {
                f.record_items(g, me, CHOICE_YES, &items);
            }
            Ok(Flow::Next)
        }
        Op::MoveCounters(m) => move_all_resume(g, me, f, m, first),
        Op::Heal(_) | Op::DamageSlot(_) | Op::PlaceCounters(_) | Op::Switch(_) => {
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
            SlotTarget::Slot(_) | SlotTarget::Each(_) => true,
        },
        Op::Switch(s) => !s.required || !slots_of(g, me, f, &switch_among(s)).is_empty(),
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
            match d.calc {
                DamageCalc::Auto => deal_or_put_damage(g, f.eff, n, slot)?,
                DamageCalc::Put => put_damage(g, f.eff, n, slot)?,
                DamageCalc::Deal => {
                    if let Some(b) = atk_base(g, f, slot) {
                        g.run_fx(Effect::DealDamage { b, damage: n })?;
                    }
                }
            }
        }
        Op::PlaceCounters(c) => {
            let n = num_m(g, me, f, &c.counters)? * 10;
            match c.cause {
                CounterCause::Effect => {
                    g.run_fx(Effect::PlaceDamageCounters { p: f.p, target: slot, damage: n, source: me })?;
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

fn switch_act(g: &mut Game, me: CardId, f: &Frame, s: &SwitchSpec, slot: SlotRef) -> R {
    let side = f.who(s.side);
    // The switch only acts on the side's own Bench.
    if slot.p as usize != side {
        return Ok(());
    }
    match s.kind {
        SwitchKind::Plain | SwitchKind::PlainBasic => crate::engine::turn::switch_pokemon(g, side, slot.s),
        SwitchKind::SilentAbilityEffect => {
            let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: f.p, power: crate::effects::PowerRef { card: me, index: 0 }, card: me, target: Some(slot) })?;
            if let Effect::EffectOfAbility { target: Some(_), .. } = fx {
                crate::engine::turn::switch_pokemon_silent(g, side, slot.s)?;
            }
            Ok(())
        }
        SwitchKind::Silent => {
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
// DamageChosen

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
    match d.calc {
        DamageCalc::Auto => deal_or_put_damage(g, f.eff, n, slot),
        DamageCalc::Put => put_damage(g, f.eff, n, slot),
        DamageCalc::Deal => {
            if let Some(b) = atk_base(g, f, slot) {
                g.run_fx(Effect::DealDamage { b, damage: n })?;
            }
            Ok(())
        }
    }
}

impl RemovePickedSpec {
    /// The candidate selector by value (the selectors are plain data).
    fn among_clone(&self) -> SlotSel {
        match &self.among {
            SlotSel::Bench(w) => SlotSel::Bench(*w),
            SlotSel::Pokemon(w) => SlotSel::Pokemon(*w),
            _ => unimplemented!("RemovePicked among: Bench or Pokemon only"),
        }
    }
}

fn remove_picked(g: &mut Game, me: CardId, f: &Frame, slot: SlotRef) -> R {
    if !occupied(g, slot) {
        return Ok(());
    }
    let (p, o) = (f.p as usize, slot.p as usize);
    if let Some((_, _, attack, _)) = attack_data(g, f.eff) {
        // An effect of the attack on that Pokémon: Mist Energy and the like prevent it.
        if attack_effect_prevented_on(g, p, o, pack_attack(attack), slot)? {
            return Ok(());
        }
    }
    move_pokemon_off_board(g, slot, crate::state::ListRef::Deck(o as u8), me)
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

fn evolve_resume(g: &mut Game, me: CardId, f: &mut Frame, e: &EvolveSpec, first: Res) -> R<Flow> {
    let p = f.who(e.chooser);
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
            evolution_prompt(g, p, name, e.stage, f.cont(me, 0x20 | t.s));
            Ok(Flow::Suspend)
        }
        0x20 => {
            let t = SlotRef::new(p, f.sub & 0x0F);
            let Some(evo) = first.cards().first().copied() else { return Ok(Flow::Next) };
            evolve_with(g, me, p, t, evo)?;
            let name = g.st.cdef(evo).name;
            if let Some(st2) = e.then_stage {
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
