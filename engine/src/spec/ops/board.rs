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
use crate::state::ListRef;
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

/// Which Benched Pokémon a switch can bring to the Active Spot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwitchAmong {
    /// Any of the side's Benched Pokémon (chosen when the switch happens, or at step D in an attack).
    Bench,
    /// The side's Benched Basic Pokémon ("1 of your opponent's Benched Basic Pokémon": Lisia's Appeal).
    BenchBasic,
    /// The Pokémon chosen by the program's last `PickSlot` (no question).
    Picked,
}

/// A switch of the Active Pokémon, as the text says it (the ChangeActive event, `engine::change_active`):
/// `change` is `Switch` ("switch your Active Pokémon with 1 of your Benched Pokémon": you choose, APR C-03),
/// `SwitchIn` ("switch in 1 of your opponent's Benched Pokémon to the Active Spot": you choose, C-05) or `SwitchOut`
/// ("switch out your opponent's Active Pokémon to the Bench; your opponent chooses the new Active Pokémon", C-04).
/// Whose Pokémon and who chooses follow from it. The text's "if you do" is `Cond::Done`.
pub struct SwitchSpec {
    pub change: crate::spec::event::ActiveChange,
    pub among: SwitchAmong,
    pub msg: &'static str,
    /// A Trainer or Ability can't be used without a Benched Pokémon to switch
    /// with (otherwise the switch is simply skipped).
    pub required: bool,
}

impl SwitchSpec {
    /// Whose Active Pokémon changes.
    pub const fn side(&self) -> Who {
        match self.change {
            crate::spec::event::ActiveChange::SwitchIn | crate::spec::event::ActiveChange::SwitchOut => Who::Opp,
            _ => Who::Me,
        }
    }
    /// Who chooses the Benched Pokémon: the side's player for a switch-out (C-04), else the program's player.
    pub const fn chooser(&self) -> Who {
        match self.change {
            crate::spec::event::ActiveChange::SwitchOut => Who::Opp,
            _ => Who::Me,
        }
    }
}

/// Heal: the RemoveCounters event (`engine::condition::heal`), whatever causes it (the frame's `Cause`).
pub struct HealSpec {
    pub target: SlotTarget,
    pub hp: Num,
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
    /// `Deal` on the opponent's Active Pokémon, `Put` elsewhere.
    Auto,
    /// The full calculation (`engine::damage::deal` with `deal`): the attacker-side modifiers, and Weakness and
    /// Resistance when the target is in an Active Spot, either player's (the attacker's damage to itself too, APR B-09);
    /// never on a Benched Pokémon (APR B-08).
    Deal,
    /// Put on the Pokémon: no attacker-side modifiers, no Weakness or Resistance (damage to Benched Pokémon).
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

/// "Put N damage counters on ...": the PlaceCounters event (`engine::damage::place`), by the frame's cause (an attack's
/// effect, an Ability, a Trainer, a Stadium, a Tool, an Energy).
pub struct PlaceCountersSpec {
    pub target: SlotTarget,
    pub counters: Num,
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
    AllFromOne { from: PickSlotSpec, to: SlotTarget },
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
    /// Salvatore: the Pokémon card of register `card` goes onto the Pokémon of `chooser` it evolves from (the
    /// chooser picks among them), as the card text says: no evolving rules are checked (also on the turn the
    /// Pokémon was played), and no Evolve effect is dispatched.
    FromRegister { chooser: Who, card: u8 },
}
/// Devolve the Pokémon (when it has an evolution card): the top card goes to `destination`.
/// An effect of the attack that effect protection can stop.
pub struct DevolveSpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
    /// The chooser picks Evolution cards from the Pokémon's stack (a ChooseCards prompt, the Basic Pokémon not
    /// selectable) and the Pokémon is devolved down to the chosen card, the cards going to `destination`;
    /// otherwise only the top card devolves.
    pub chooser: Option<Who>,
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
    /// The bottom card of the stack is replaced (otherwise the top card).
    pub bottom: bool,
}
/// This Pokémon (the slot `target`) switches with the Active Pokémon when it is on the Bench.
pub struct SwitchWithActiveSpec {
    pub target: SlotExpr,
}
/// Put a Pokémon and all cards attached to it into a zone: the LeavePlay event (`engine::knockout::leave_play`), by the
/// frame's cause (an attack's removal is an effect of the attack, which "prevent all effects of attacks" stops).
pub struct RemoveFromPlaySpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
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
    pub gate: Gate,
    pub when: Cond,
}

/// "Knock Out ..." / "this Pokémon is Knocked Out" by an effect (`engine::knockout::by_effect`): recorded, then Knocked
/// Out at the next state check (user decision D1); its owner's opponent takes the Prize cards.
pub struct KnockOutSpec {
    pub target: SlotExpr,
    pub when: Cond,
}

// ---------------------------------------------------------------------------
// Helpers

pub(crate) fn encode(s: SlotRef) -> u8 {
    s.p << 4 | s.s
}

pub(crate) fn decode(b: u8) -> SlotRef {
    SlotRef::new((b >> 4) as usize, b & 15)
}

pub(crate) fn occupied(g: &Game, s: SlotRef) -> bool {
    !g.st.slot(s.p as usize, s.s).cards.is_empty()
}

/// The AttackEffect a frame's events are effects of, with the attack's player, opponent, attack and attacking
/// Pokémon. An attack's program: its own effect (`f.eff`, retained until the attack finishes). A Trainer used as
/// the effect of an attack (Look-Alike Show): the attack in progress (`Game::last_attack`), never `f.eff`, which is
/// the Trainer's own effect (released when its program suspends for a prompt, its slot possibly reused after the
/// resume): the Supporter's effect is the attack's effect (id2225, id2226).
pub(crate) fn frame_attack(g: &Game, f: &Frame) -> Option<(crate::effects::EffId, u8, u8, crate::state::AttackRef, SlotRef)> {
    if f.via_attack {
        let la = g.last_attack?;
        if (la.effect as usize) < g.fx.len() && matches!(*g.e(la.effect), Effect::Attack { attack, .. } if attack == la.attack) {
            return Some((la.effect, la.p, 1 - la.p, la.attack, la.source));
        }
        return None;
    }
    // A program resumed after a prompt may outlive the effect that started it (a step 7 trigger after its
    // AttackTrigger is released): no attack then. (g0500007065 crashed here when a Trainer used through an attack
    // read its released effect; that case is the branch above now.)
    if f.eff as usize >= g.fx.len() {
        return None;
    }
    let (p, opp, attack, source) = attack_data(g, f.eff)?;
    Some((f.eff, p, opp, attack, source))
}

pub(crate) fn atk_base(g: &Game, f: &Frame, target: SlotRef) -> Option<AtkBase> {
    // A step 7 trigger acts for the attack it belongs to.
    if !f.via_attack && (f.eff as usize) < g.fx.len() {
        if let Effect::AttackTrigger { attack_effect, p, opp, attack, source, .. } = *g.e(f.eff) {
            return Some(AtkBase { attack_effect, player: p, opponent: opp, attack, source, target, cause: f.cause });
        }
    }
    let (attack_effect, p, opp, attack, source) = frame_attack(g, f)?;
    Some(AtkBase { attack_effect, player: p, opponent: opp, attack, source, target, cause: f.cause })
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
        SlotSel::Cancelable(inner) => return sel_types(inner),
    }
    v
}

/// The owner of the Pokémon a selector ranges over.
fn sel_owner(sel: &SlotSel, f: &Frame) -> usize {
    match sel {
        SlotSel::One(SlotExpr::Active(w)) | SlotSel::Bench(w) | SlotSel::Pokemon(w) | SlotSel::PokemonBenchFirst(w) => f.who(*w),
        SlotSel::One(SlotExpr::This | SlotExpr::Marked(_)) => f.p as usize,
        SlotSel::One(SlotExpr::Picked) => (f.slot >> 4) as usize,
        SlotSel::One(SlotExpr::Attached) => (f.attached_to >> 4) as usize,
        // The program's own player's view of the opponent's Attacking Pokémon (the event's cause).
        SlotSel::One(SlotExpr::CausePokemon) => 1 - f.p as usize,
        SlotSel::Filtered(inner, _) => sel_owner(inner, f),
        SlotSel::Cancelable(inner) => sel_owner(inner, f),
    }
}

/// Ask the chooser to pick one of `cands` (resumed at `sub`).
pub(crate) fn ask(g: &mut Game, me: CardId, f: &Frame, pick: &PickSlotSpec, cands: &[SlotRef], sub: u8) {
    ask_range(g, me, f, pick, cands, 1, 1, sub)
}

/// Ask the chooser to pick between `min` and `max` of `cands` (resumed at `sub`).
pub(crate) fn ask_range(g: &mut Game, me: CardId, f: &Frame, pick: &PickSlotSpec, cands: &[SlotRef], min: u8, max: u8, sub: u8) {
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
        PromptKind::ChoosePokemon { player_type, slots, min, max, allow_cancel: matches!(pick.among, SlotSel::Cancelable(_)), blocked },
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
pub(crate) fn candidates(g: &mut Game, me: CardId, f: &Frame, pick: &PickSlotSpec) -> R<SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>> {
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
                let dst = zone_ref(f, r.destination);
                crate::engine::knockout::leave_play(g, slot, dst, f.cause, me)?;
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
            crate::engine::knockout::by_effect(g, slot, f.cause)?;
            Ok(Flow::Next)
        }
        Op::Switch(s) => switch_exec(g, me, f, s),
        Op::Evolve(EvolveSpec { how: EvolveHow::RareCandy }) => {
            let p = f.p as usize;
            let stage2 = stage2_in_hand(g, p);
            let mut blocked: TargetList = SVec::new();
            for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.cdef(c).stage == Stage::Basic as u8 && candy_can_evolve(g, f.cause, SlotRef::new(p, s), &stage2)? {
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
        Op::Evolve(EvolveSpec { how: EvolveHow::FromRegister { chooser, card } }) => evolve_reg_exec(g, me, f, *chooser, *card),
        Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot, card } }) => {
            let (Some(slot), Some(&card)) = (slot_of(g, me, f, *slot), reg_list(g, f, *card).first()) else { return Ok(Flow::Next) };
            if g.st.locate(card).is_none() {
                return Ok(Flow::Next);
            }
            // Put onto the Pokémon by the effect: the effect path.
            crate::engine::enter::evolve(g, card, slot, super::super::event::EvolvePath::Effect, crate::engine::enter::Reach::Next, f.cause)?;
            Ok(Flow::Next)
        }
        Op::Devolve(d) if d.chooser.is_some() => devolve_exec(g, me, f, d),
        Op::Devolve(dv) => {
            let Some(slot) = slot_of(g, me, f, dv.slot) else { return Ok(Flow::Next) };
            let (p, s) = (slot.p as usize, slot.s);
            if occupied(g, slot) && g.st.slot_pokemons(p, s).len() > 1 {
                // The Devolve event by the frame's cause (an attack's: "prevent all effects of attacks" stops it).
                crate::engine::enter::devolve(g, slot, 1, zone_ref(f, dv.destination), f.cause)?;
            }
            Ok(Flow::Next)
        }
        Op::SwapPokemonCard(sw) if sw.bottom => swap_bottom_exec(g, me, f, sw),
        Op::SwapPokemonCard(sw) => {
            // The chosen card goes onto this Pokémon's slot, this card leaves for `into`; it is the
            // same Pokémon (id2372): the card-bound facts move to the new card (the Swap event).
            let new = reg_list(g, f, sw.cards).first().copied();
            let slot = slot_of(g, me, f, sw.slot);
            if let (Some(new), Some(slot)) = (new, slot) {
                let (p, s) = (slot.p as usize, slot.s);
                let Some(old) = g.st.slot_pokemon(p, s) else { return Ok(Flow::Next) };
                if g.st.locate(new).is_some() {
                    let place = if sw.keep_index { crate::engine::enter::SwapPlace::OldIndex } else { crate::engine::enter::SwapPlace::Top };
                    crate::engine::enter::swap(g, slot, old, new, zone_ref(f, sw.into), place, me, f.cause)?;
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
            // "Switch it with your Active Pokémon" (Iron Leaves ex's Rapid Vernier): the player's own switch.
            f.done = false;
            if let Some(s) = slot_of(g, me, f, w.target) {
                let p = s.p as usize;
                if g.st.players[p].bench_index_of(s.s).is_some() {
                    let c = crate::engine::change_active::ChangeActiveView::of(g, p, Some(s.s), crate::spec::event::ActiveChange::Switch, f.cause);
                    f.done = crate::engine::change_active::change_active(g, c)?;
                }
            }
            Ok(Flow::Next)
        }
        Op::Evolve(EvolveSpec { how: EvolveHow::FromDeck { chooser, .. } }) => {
            let p = f.who(*chooser);
            let (_, blocked) = evolve_targets(g, p, f.cause)?;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            let id = g.player_id(p);
            g.prompt(id, "CHOOSE_POKEMON_TO_EVOLVE", PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked }, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::SpreadCounters(s) => spread_exec(g, me, f, s),
        Op::MoveCounters(m) => match &m.kind {
            MoveCountersKind::AllFromOne { .. } => move_all_exec(g, me, f, m),
            MoveCountersKind::AnyAmong { who } => move_any_exec(g, me, f, *who),
            MoveCountersKind::MineToOpp { max } => mine_to_opp_exec(g, me, f, *max),
        },
        Op::EachSlot(e) => each_exec(g, me, f, e),
        Op::ChoiceDamage(_) => Ok(Flow::Next),
        _ => unimplemented!("spec op not implemented yet (ops/board.rs)"),
    }
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
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
            f.done = false;
            if let Some(slot) = first.slots().first().copied() {
                switch_act(g, f, s, slot)?;
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
                if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
                    return Ok(Flow::Next);
                }
                f.slot = encode(target);
                let mut opts = ChooseCardsOpts::new(1, 1, false);
                let hand: Vec<CardId> = g.st.players[p].hand.iter().collect();
                for (i, c) in hand.iter().enumerate() {
                    let d = g.st.cdef(*c);
                    let stage2 = d.is_pokemon() && d.stage == Stage::Stage2 as u8;
                    if stage2 && !candy_can_evolve(g, f.cause, target, &[*c])? {
                        opts.blocked.push(i as u8);
                    }
                }
                let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Stage2 as u8), ..Filter::none() };
                choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", crate::state::ListRef::Hand(p as u8), filter, opts, f.cont(me, 2));
                return Ok(Flow::Suspend);
            }
            if let Some(c) = first.cards().first().copied() {
                let target = decode(f.slot);
                // It counts as evolving (id1045), and as playing the card from the hand (id285, id1998): the
                // rule path, skipping the Stage 1.
                crate::engine::enter::evolve(g, c, target, super::super::event::EvolvePath::Rule, crate::engine::enter::Reach::SkipStage1, f.cause)?;
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
        Op::Evolve(EvolveSpec { how: EvolveHow::FromRegister { card, .. } }) => evolve_reg_resume(g, me, f, *card, first),
        Op::Devolve(d) => devolve_resume(g, f, d, first),
        Op::EachSlot(e) => {
            let slots: Vec<SlotRef> = first.slots().to_vec();
            each_act(g, me, f, e, &slots)?;
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

/// Step D: the choice of an attack effect, asked before the damage.
pub(crate) fn choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
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
            if cands.is_empty() || switch_out_refused(g, f, s)? {
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            ask(g, me, f, &switch_pick(s), cands.as_slice(), 1);
            Ok(Flow::Suspend)
        }
        Op::MoveCounters(m) if matches!(m.kind, MoveCountersKind::AllFromOne { .. }) => move_all_exec(g, me, f, m),
        Op::PickSlot(pick) => {
            // A fixed Pokémon is just selected when the effect is carried out (nothing to ask or
            // record; the step D steps after it read it).
            if matches!(pick.among, SlotSel::One(_)) {
                let cands = candidates(g, me, f, pick)?;
                f.slot = cands.as_slice().first().map(|s| encode(*s)).unwrap_or(NONE);
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
        Op::EachSlot(e) if e.choose.is_some() => each_ask(g, me, f, e, true),
        Op::ChoiceDamage(c) => {
            let n = match c.reg {
                Some(r) => reg_list(g, f, r).len() as i32,
                None => 1,
            };
            if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
                match c.op {
                    DamageOp::Add => *damage += c.per * n,
                    DamageOp::Set => *damage = c.per * n,
                }
            }
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::EachSlot(_) => {
            let items: Vec<u8> = first.slots().iter().map(|s| encode(*s)).collect();
            f.record_items(g, me, CHOICE_YES, &items);
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
        // A pick needs a Pokémon to choose.
        Op::PickSlot(p) => !slots_of(g, me, f, &p.among).is_empty(),
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// Acting on a chosen Pokémon

fn act(g: &mut Game, me: CardId, f: &Frame, op: &Op, slot: SlotRef) -> R {
    match op {
        Op::Heal(h) => {
            let n = num_m(g, me, f, &h.hp)?;
            crate::engine::condition::heal(g, slot, n, f.cause)?;
            if h.clear_conditions {
                crate::engine::condition::recover_all(g, slot, f.cause, &[])?;
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
            crate::engine::damage::place(g, slot, n, f.cause)?;
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
            inflict_on(g, me, f, slot, cs)?;
            Ok(Flow::Next)
        }
        ConditionChange::RemoveAll => {
            crate::engine::condition::recover_all(g, slot, f.cause, &[])?;
            Ok(Flow::Next)
        }
        ConditionChange::RemoveChosen => {
            let conds: Vec<u8> = g.st.slot(p, s).special_conditions.as_slice().to_vec();
            if conds.len() == 1 {
                crate::engine::condition::remove(g, slot, SpecialCondition::from_u8(conds[0]), f.cause)?;
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
    crate::engine::condition::remove(g, slot, SpecialCondition::from_u8(choice as u8), f.cause)?;
    Ok(())
}

/// The Pokémon in `slot` is now affected by `cs`, by the frame's cause: one GainCondition each
/// (`engine::condition::gain`), which the preventions over it stop whatever the cause (Slowpoke's Dopey Face vs
/// Lisia's Appeal; "prevent all effects of attacks" against an attack's).
fn inflict_on(g: &mut Game, _me: CardId, f: &Frame, slot: SlotRef, cs: &[SpecialCondition]) -> R {
    for x in cs {
        crate::engine::condition::gain(g, slot, *x, f.cause)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Switching

/// The Pokémon a switch can bring to the Active Spot.
fn switch_among(s: &SwitchSpec) -> SlotSel {
    match s.among {
        // Blocked: a Pokémon that is not Basic (a slot without a Pokémon card, a Fossil, is not).
        SwitchAmong::BenchBasic => match s.side() {
            Who::Me => SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Basic),
            Who::Opp => SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Basic),
        },
        _ => SlotSel::Bench(s.side()),
    }
}

fn switch_pick(s: &SwitchSpec) -> PickSlotSpec {
    PickSlotSpec { chooser: s.chooser(), among: switch_among(s), msg: s.msg }
}

/// The ChangeActive a switch op makes, with the Benched Pokémon `to` (`None`: not chosen yet).
fn switch_change(g: &Game, f: &Frame, s: &SwitchSpec, to: Option<crate::state::SlotId>) -> crate::engine::change_active::ChangeActiveView {
    crate::engine::change_active::ChangeActiveView::of(g, f.who(s.side()), to, s.change, f.cause)
}

/// A switch-out is done to the Active Pokémon (APR C-04, id2025): when the change itself is refused (Mist Energy on
/// the Active Pokémon against an attack's switch-out), the opponent isn't asked to choose.
fn switch_out_refused(g: &mut Game, f: &Frame, s: &SwitchSpec) -> R<bool> {
    if s.change != crate::spec::event::ActiveChange::SwitchOut {
        return Ok(false);
    }
    let c = switch_change(g, f, s, None);
    crate::engine::change_active::refused(g, &c)
}

fn switch_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SwitchSpec) -> R<Flow> {
    f.done = false;
    if s.among == SwitchAmong::Picked {
        if let Some(slot) = slot_of(g, me, f, SlotExpr::Picked) {
            if occupied(g, slot) {
                switch_act(g, f, s, slot)?;
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
            switch_act(g, f, s, slot)?;
        }
        return Ok(Flow::Next);
    }
    let cands = slots_of(g, me, f, &switch_among(s));
    if cands.is_empty() || switch_out_refused(g, f, s)? {
        return Ok(Flow::Next);
    }
    ask(g, me, f, &switch_pick(s), cands.as_slice(), 1);
    Ok(Flow::Suspend)
}

/// The switch with the chosen Benched Pokémon: the ChangeActive event (`engine::change_active`), which the locks and
/// preventions can refuse (a switch-in by an opponent's attack or Ability of a Pokémon protected from them: APR C-05,
/// id2155, JP Q&A on Hariyama's Heave-Ho Catcher); `f.done` says whether it happened.
fn switch_act(g: &mut Game, f: &mut Frame, s: &SwitchSpec, slot: SlotRef) -> R {
    let side = f.who(s.side());
    // The Pokémon that leaves the Active Spot is the picked slot afterwards (for effects on it).
    f.slot = encode(SlotRef::new(side, g.st.players[side].active));
    // The switch only acts on the side's own Bench.
    if slot.p as usize != side {
        return Ok(());
    }
    let c = switch_change(g, f, s, Some(slot.s));
    f.done = crate::engine::change_active::change_active(g, c)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Spreading and moving counters

fn spread_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SpreadCountersSpec) -> R<Flow> {
    let p = f.who(s.chooser);
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (sl, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let hp = crate::derived::hp(g, p, sl)?;
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
    if attack_data(g, f.eff).is_none() {
        return Ok(());
    }
    let map: SVec<(CardTarget, i32), 16> = match first {
        Res::DamageMap(m) => m,
        _ => SVec::new(),
    };
    if let Effect::Attack { damage: d, .. } = g.e_mut(f.eff) {
        *d = 0;
    }
    for (t, damage) in map.iter() {
        let target = get_target(&g.st, p, *t)?;
        crate::engine::damage::place(g, target, *damage, f.cause)?;
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
        let hp = crate::derived::hp(g, owner, sl)?;
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
    if attack_data(g, f.eff).is_none() {
        return Ok(());
    }
    // One action: every counter the player moves, one at a time in the order given (the MoveCounters event).
    let mut pairs: Vec<(SlotRef, SlotRef, i32)> = Vec::new();
    for (from, to) in damage_transfers(transfers.as_slice()) {
        pairs.push((get_target(&g.st, p, from)?, get_target(&g.st, p, to)?, 10));
    }
    move_in_batches(g, &pairs, f.cause)
}

/// Move the counters of `pairs` as one action (`engine::damage::move_counters`): consecutive moves between the same two
/// Pokémon are one pair (the counters move one at a time either way); an event holds up to 16 pairs.
fn move_in_batches(g: &mut Game, pairs: &[(SlotRef, SlotRef, i32)], cause: crate::cause::Cause) -> R {
    let mut merged: Vec<(SlotRef, SlotRef, i32)> = Vec::new();
    for &(a, b, hp) in pairs {
        match merged.last_mut() {
            Some(l) if l.0 == a && l.1 == b => l.2 += hp,
            _ => merged.push((a, b, hp)),
        }
    }
    for chunk in merged.chunks(16) {
        crate::engine::damage::move_counters(g, chunk, cause)?;
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
    // The destination: asked after the source, or fixed (nothing is asked for it).
    let (src, tgt) = match to {
        SlotTarget::Slot(e) => {
            let Some(dst) = slot_of(g, me, f, *e) else { return Ok(Flow::Next) };
            (picked, dst)
        }
        SlotTarget::Pick(to) => {
            if f.sub < 0x80 {
                let cands = candidates(g, me, f, to)?;
                ask(g, me, f, to, cands.as_slice(), 0x80 | encode(picked));
                return Ok(Flow::Suspend);
            }
            (decode(f.sub & 0x7F), picked)
        }
    };
    if f.phase == Phase::Choices {
        f.record_items(g, me, CHOICE_YES, &[encode(src), encode(tgt)]);
    } else {
        move_all_act(g, f, src, tgt)?;
    }
    Ok(Flow::Next)
}

/// All the counters on `src` move to `tgt` (one MoveCounters event).
fn move_all_act(g: &mut Game, f: &Frame, src: SlotRef, tgt: SlotRef) -> R {
    let move_damage = g.st.slot(src.p as usize, src.s).damage;
    if move_damage <= 0 || attack_data(g, f.eff).is_none() {
        return Ok(());
    }
    crate::engine::damage::move_counters(g, &[(src, tgt, move_damage)], f.cause)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Short forms for spec files

pub const OPP_ACTIVE: SlotExpr = SlotExpr::Active(Who::Opp);
pub const MY_ACTIVE: SlotExpr = SlotExpr::Active(Who::Me);

/// The opponent's Active Pokémon is now affected by these Special Conditions (by the frame's cause).
pub const fn inflict(cs: &'static [SpecialCondition]) -> Op {
    Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(cs), gate: Gate::None, when: Cond::True })
}

/// "This Pokémon also does N damage to itself."
pub const fn self_damage(hp: i32) -> Op {
    Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(hp), target_damage_mul: 0, calc: DamageCalc::Deal, when: Cond::True })
}

/// "Switch this Pokémon with 1 of your Benched Pokémon."
pub const fn switch_self() -> Op {
    Op::Switch(SwitchSpec { change: crate::spec::event::ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false })
}

/// Heal damage from this Pokémon.
pub const fn heal_active(hp: i32) -> Op {
    Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(hp), clear_conditions: false })
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
        let hp = crate::derived::hp(g, p, s)?;
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
    let _ = me;
    // One use of the Ability is one action (id67, id2152): up to `max` counters, one at a time, in the order given.
    let limit = max as i32 * 10;
    let mut pairs: Vec<(SlotRef, SlotRef, i32)> = Vec::new();
    let mut total = 0;
    for (from, to) in damage_transfers(transfers.as_slice()) {
        if total >= limit {
            break;
        }
        pairs.push((get_target(&g.st, p, from)?, get_target(&g.st, p, to)?, 10.min(limit - total)));
        total += 10;
    }
    move_in_batches(g, &pairs, f.cause)
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
                crate::engine::damage::place(g, slot, damage, f.cause)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rare Candy (S3 agent 3; its checks are the Evolve event's: `candy_can_evolve`)

pub fn stage2_in_hand(g: &Game, p: usize) -> Vec<CardId> {
    g.st.players[p].hand.iter().filter(|c| {
        let d = g.st.cdef(*c);
        d.is_pokemon() && d.stage == Stage::Stage2 as u8
    }).collect()
}

/// `n` damage to the Pokémon the way `calc` says.
fn damage_by(g: &mut Game, f: &Frame, calc: DamageCalc, n: i32, slot: SlotRef) -> R {
    match calc {
        DamageCalc::Auto => deal_or_put_damage(g, f.eff, n, slot)?,
        DamageCalc::Put => put_damage(g, f.eff, n, slot)?,
        DamageCalc::Deal => {
            if let Some(b) = atk_base(g, f, slot) {
                crate::engine::damage::deal(g, b, n, true)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Evolve (Grand Tree): the Evolve event's effect path, with the card's own restriction

fn evolves_from_any(name: &str) -> bool {
    crate::gen::evolutions::ALL_EVOLUTIONS.iter().any(|(_, from)| *from == name)
}

/// Grand Tree's limits on the Pokémon in `t` ("Players can't evolve a Basic Pokémon during their first turn or
/// a Basic Pokémon that was put into play this turn"): the card's own restriction (`CardSpec::restricts`, which
/// no permission lifts: official JP Q&A 2026-10-09), asked before the card is chosen
/// (`engine::enter::evolve_limits`).
fn from_deck_refused(g: &mut Game, t: SlotRef, cause: crate::cause::Cause) -> R<bool> {
    let Some(v) = crate::engine::enter::evolve_view(g, None, t, super::super::event::RulesZone::Deck, super::super::event::EvolvePath::Effect, cause) else { return Ok(true) };
    Ok(crate::engine::enter::evolve_limits(g, &v)?.is_some())
}

/// Whether some Basic Pokémon of `p` can evolve now (the cause card's restrictions allow it) into a card the
/// game knows, and the Pokémon prompt's blocked targets.
pub(crate) fn evolve_targets(g: &mut Game, p: usize, cause: crate::cause::Cause) -> R<(bool, TargetList)> {
    let mut any = false;
    let mut blocked: TargetList = SVec::new();
    for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Basic as u8 || from_deck_refused(g, SlotRef::new(p, s), cause)? {
            blocked.push(t);
            continue;
        }
        if evolves_from_any(g.st.cdef(c).name) {
            any = true;
        }
    }
    Ok((any, blocked))
}

/// Can the effect (`cause`) evolve the Pokémon in `t` with `card` from `source`: it evolves from the Pokémon,
/// and the Evolve event's checks allow it (`enter::check_evolve`: the locks, the card's own
/// restrictions). An effect offers only cards it can put onto the Pokémon.
fn effect_can_evolve(g: &mut Game, t: SlotRef, card: CardId, source: super::super::event::RulesZone, cause: crate::cause::Cause) -> R<bool> {
    use crate::engine::enter::{check_evolve, evolve_view, evolves_into, Reach};
    let Some(base) = g.st.slot_pokemon(t.p as usize, t.s) else { return Ok(false) };
    if !evolves_into(g, base, card, Reach::Next) {
        return Ok(false);
    }
    let Some(v) = evolve_view(g, Some(card), t, source, super::super::event::EvolvePath::Effect, cause) else { return Ok(false) };
    Ok(check_evolve(g, &v, Reach::Next).is_ok())
}

/// Deck prompt for an evolution (`stage`) of the Pokémon in `t`, blocking the deck Pokémon the effect can't put
/// onto it ([`effect_can_evolve`]).
fn evolution_prompt(g: &mut Game, p: usize, t: SlotRef, stage: Stage, cause: crate::cause::Cause, cont: crate::game::Cont) -> R {
    let Some(base) = g.st.slot_pokemon(t.p as usize, t.s) else { return Ok(()) };
    let mut blocked = Blocked::default();
    let deck: Vec<CardId> = g.st.players[p].deck.iter().collect();
    for (i, c) in deck.iter().copied().enumerate() {
        if g.st.cdef(c).is_pokemon() && !effect_can_evolve(g, t, c, super::super::event::RulesZone::Deck, cause)? {
            blocked.push(i as u8);
        }
    }
    let mut opts = ChooseCardsOpts::new(1, 1, true);
    opts.blocked = blocked;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(stage as u8), evolves_from: Some(g.st.cdef(base).name), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), filter, opts, cont);
    Ok(())
}

/// An effect evolves the Pokémon with a card from the deck: the Evolve event's effect path (the card isn't
/// played from the hand: no lock or trigger on that applies; id1133, id2037). `false`: the event was refused.
fn evolve_with(g: &mut Game, t: SlotRef, card: CardId, cause: crate::cause::Cause) -> R<bool> {
    crate::engine::enter::evolve(g, card, t, super::super::event::EvolvePath::Effect, crate::engine::enter::Reach::Next, cause)
}

fn evolve_resume(g: &mut Game, me: CardId, f: &mut Frame, chooser: Who, stage: Stage, then_stage: Option<Stage>, first: Res) -> R<Flow> {
    let p = f.who(chooser);
    match f.sub & 0xF0 {
        0x00 => {
            // The Pokémon was chosen.
            let Some(t) = first.slots().first().copied() else { return Ok(Flow::Next) };
            let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) else { return Ok(Flow::Next) };
            if g.st.cdef(c).stage != Stage::Basic as u8 || from_deck_refused(g, t, f.cause)? {
                return Ok(Flow::Next);
            }
            evolution_prompt(g, p, t, stage, f.cause, f.cont(me, 0x20 | t.s))?;
            Ok(Flow::Suspend)
        }
        0x20 => {
            let t = SlotRef::new(p, f.sub & 0x0F);
            let Some(evo) = first.cards().first().copied() else { return Ok(Flow::Next) };
            // "If that Pokémon was evolved in this way": the Stage 2 search needs the first evolving.
            if !evolve_with(g, t, evo, f.cause)? {
                return Ok(Flow::Next);
            }
            let name = g.st.cdef(evo).name;
            if let Some(st2) = then_stage {
                if evolves_from_any(name) {
                    evolution_prompt(g, p, t, st2, f.cause, f.cont(me, 0x30 | t.s))?;
                    return Ok(Flow::Suspend);
                }
            }
            Ok(Flow::Next)
        }
        _ => {
            let t = SlotRef::new(p, f.sub & 0x0F);
            if let Some(c) = first.cards().first().copied() {
                evolve_with(g, t, c, f.cause)?;
            }
            Ok(Flow::Next)
        }
    }
}

/// Can Rare Candy (`cause`) evolve the Basic Pokémon in `t` with one of `stage2` (cards in the hand)? The Evolve
/// event's checks on the rule path, skipping the Stage 1: the locks on evolving from the hand (id1133, id285,
/// id1998), the rule's limits and Rare Candy's own restriction (no permission lifts it: id1144, id1815).
pub(crate) fn candy_can_evolve(g: &mut Game, cause: crate::cause::Cause, t: SlotRef, stage2: &[CardId]) -> R<bool> {
    use crate::engine::enter::{check_evolve, evolve_view, evolves_into, Reach};
    let Some(base) = g.st.slot_pokemon(t.p as usize, t.s) else { return Ok(false) };
    for &c in stage2 {
        if !evolves_into(g, base, c, Reach::SkipStage1) {
            continue;
        }
        let Some(v) = evolve_view(g, Some(c), t, super::super::event::RulesZone::Hand, super::super::event::EvolvePath::Rule, cause) else { return Ok(false) };
        if check_evolve(g, &v, Reach::SkipStage1).is_ok() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `canUseRareCandy`: some Basic Pokémon of `p` can evolve with a Stage 2 card in the hand
/// ([`candy_can_evolve`]); `me` is the Rare Candy card.
pub fn rare_candy_usable(g: &mut Game, me: CardId, p: usize) -> R<bool> {
    let stage2 = stage2_in_hand(g, p);
    if stage2.is_empty() {
        return Ok(false);
    }
    let cause = crate::cause::Cause::of_trainer(g, me, p as u8);
    for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage == Stage::Basic as u8 && candy_can_evolve(g, cause, SlotRef::new(p, s), &stage2)? {
            return Ok(true);
        }
    }
    Ok(false)
}

// ---------------------------------------------------------------------------
// S3-4: EachSlot, ChoiceDamage, Salvatore's evolution, picked devolution, swapped Pokémon card

/// Choose between `min` and `max` Pokémon first (an attack's choice is made at step D).
pub struct ChooseN {
    pub chooser: Who,
    pub min: Num,
    pub max: Num,
    pub msg: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EachWhat {
    /// Damage to each (by `calc`).
    Damage(DamageCalc),
    /// Damage counters on each (PlaceCounters, by the frame's cause).
    Counters,
    /// Each Pokémon and all cards attached to it are shuffled into its owner's deck (an
    /// effect of the attack, which effect protection stops).
    ShuffleIntoDeck,
}

/// Do the same to each of several Pokémon, all of a selection or the ones the chooser picks.
pub struct EachSlotSpec {
    pub among: SlotSel,
    pub choose: Option<ChooseN>,
    pub what: EachWhat,
    /// HP of damage, or damage counters.
    pub amount: Num,
    /// Plus this many HP for each HP of damage already on the target.
    pub per_damage: i32,
    /// Skip Pokémon without damage counters.
    pub only_damaged: bool,
    pub when: Cond,
}

impl EachSlotSpec {
    pub const DEFAULT: EachSlotSpec =
        EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: None, what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(0), per_damage: 0, only_damaged: false, when: Cond::True };
}

/// Step D: the attack's damage follows what was chosen just before (the cards of register
/// `reg`, or a flat amount when there is none): "N for each card", "N more for each card".
pub struct ChoiceDamageSpec {
    pub reg: Option<u8>,
    pub op: DamageOp,
    pub per: i32,
}

/// The prompt of an `EachSlot` choice: the selection's side and slot kinds decide what is listed.
fn each_pick(e: &EachSlotSpec, c: &ChooseN) -> PickSlotSpec {
    PickSlotSpec { chooser: c.chooser, among: clone_sel(&e.among), msg: c.msg }
}

/// A selection that stands in for `sel` in a prompt (the same shape, never evaluated).
fn clone_sel(sel: &SlotSel) -> SlotSel {
    match sel {
        SlotSel::One(e) => SlotSel::One(*e),
        SlotSel::Bench(w) => SlotSel::Bench(*w),
        SlotSel::Pokemon(w) => SlotSel::Pokemon(*w),
        SlotSel::PokemonBenchFirst(w) => SlotSel::PokemonBenchFirst(*w),
        SlotSel::Filtered(inner, _) => clone_sel(inner),
        SlotSel::Cancelable(inner) => clone_sel(inner),
    }
}

/// Ask for the Pokémon of an `EachSlot` with `choose` (resumed at 1).
fn each_ask(g: &mut Game, me: CardId, f: &mut Frame, e: &EachSlotSpec, choice: bool) -> R<Flow> {
    let Some(c) = &e.choose else { return Ok(Flow::Next) };
    if !cond_m(g, me, f, &e.when)? {
        if choice {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    let cands = slots_m(g, me, f, &e.among)?;
    if cands.is_empty() {
        if choice {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    let max = num_m(g, me, f, &c.max)?.clamp(0, 255);
    let min = num_m(g, me, f, &c.min)?.clamp(0, max.min(cands.len() as i32));
    ask_range(g, me, f, &each_pick(e, c), cands.as_slice(), min as u8, max as u8, 1);
    Ok(Flow::Suspend)
}

fn each_exec(g: &mut Game, me: CardId, f: &mut Frame, e: &EachSlotSpec) -> R<Flow> {
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer != CHOICE_NONE {
            let slots: Vec<SlotRef> = c.items[..c.len as usize].iter().map(|b| decode(*b)).collect();
            each_act(g, me, f, e, &slots)?;
        }
        return Ok(Flow::Next);
    }
    if e.choose.is_some() {
        return each_ask(g, me, f, e, false);
    }
    if !cond_m(g, me, f, &e.when)? {
        return Ok(Flow::Next);
    }
    let slots: Vec<SlotRef> = slots_m(g, me, f, &e.among)?.iter().copied().collect();
    each_act(g, me, f, e, &slots)?;
    Ok(Flow::Next)
}

fn each_act(g: &mut Game, me: CardId, f: &Frame, e: &EachSlotSpec, slots: &[SlotRef]) -> R {
    for slot in slots {
        let slot = *slot;
        if !occupied(g, slot) {
            continue;
        }
        let target_damage = g.st.slot(slot.p as usize, slot.s).damage;
        if e.only_damaged && target_damage <= 0 {
            continue;
        }
        match e.what {
            EachWhat::Damage(calc) => {
                let n = num_m(g, me, f, &e.amount)? + e.per_damage * target_damage;
                damage_by(g, f, calc, n, slot)?;
            }
            EachWhat::Counters => {
                let n = num_m(g, me, f, &e.amount)? * 10 + e.per_damage * target_damage;
                crate::engine::damage::place(g, slot, n, f.cause)?;
            }
            EachWhat::ShuffleIntoDeck => {
                // The LeavePlay event by the frame's cause (an attack's effect: Mist Energy and the like prevent it).
                if !crate::engine::knockout::leave_play(g, slot, crate::state::ListRef::Deck(slot.p), f.cause, me)? {
                    continue;
                }
                let id = g.player_id(slot.p as usize);
                g.prompt(id, "", PromptKind::ShuffleDeck, crate::game::Cont::ShuffleApplyNoWait { p: slot.p });
            }
        }
    }
    Ok(())
}

fn evolve_reg_exec(g: &mut Game, me: CardId, f: &mut Frame, chooser: Who, card: u8) -> R<Flow> {
    let Some(evo) = reg_list(g, f, card).first().copied() else { return Ok(Flow::Next) };
    let Some((_, source)) = crate::engine::enter::source_of(g, evo) else { return Ok(Flow::Next) };
    let owner = f.who(chooser);
    // The Pokémon the effect can put the card onto (the Evolve event's checks: [`effect_can_evolve`]).
    let mut cands: Vec<SlotRef> = Vec::new();
    for (s, _, _) in for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().copied() {
        let t = SlotRef::new(owner, s);
        if effect_can_evolve(g, t, evo, source, f.cause)? {
            cands.push(t);
        }
    }
    if cands.is_empty() {
        return Ok(Flow::Next);
    }
    let pick = PickSlotSpec { chooser, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON_TO_EVOLVE" };
    ask(g, me, f, &pick, &cands, 1);
    Ok(Flow::Suspend)
}

fn evolve_reg_resume(g: &mut Game, _me: CardId, f: &mut Frame, card: u8, first: Res) -> R<Flow> {
    let Some(t) = first.slots().first().copied() else { return Ok(Flow::Next) };
    let Some(evo) = reg_list(g, f, card).first().copied() else { return Ok(Flow::Next) };
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        return Ok(Flow::Next);
    }
    if g.st.locate(evo).is_some() {
        // Put onto the Pokémon by the effect (no limit: "you can use this card on a Pokémon ... put into play
        // this turn").
        crate::engine::enter::evolve(g, evo, t, super::super::event::EvolvePath::Effect, crate::engine::enter::Reach::Next, f.cause)?;
    }
    Ok(Flow::Next)
}

fn devolve_exec(g: &mut Game, me: CardId, f: &mut Frame, d: &DevolveSpec) -> R<Flow> {
    let Some(t) = slot_of(g, me, f, d.slot) else { return Ok(Flow::Next) };
    let (tp, ts) = (t.p as usize, t.s);
    let stack = g.st.slot_pokemons(tp, ts);
    if stack.is_empty() {
        return Ok(Flow::Next);
    }
    // The Basic can't be put into the hand: it is blocked by index.
    let basic = stack.get(0).copied();
    let mut opts = ChooseCardsOpts::new(1, 1, false);
    for (i, c) in g.st.slot(tp, ts).cards.iter().enumerate() {
        if Some(c) == basic {
            opts.blocked.push(i as u8);
        }
    }
    let chooser = f.who(d.chooser.unwrap_or(Who::Me));
    choose_cards(g, chooser, "CHOOSE_POKEMON_TO_PICK_UP", crate::state::ListRef::Slot(t.p, t.s), Filter::super_type(SuperType::Pokemon), opts, f.cont(me, 0x80 | encode(t)));
    Ok(Flow::Suspend)
}

fn devolve_resume(g: &mut Game, f: &Frame, d: &DevolveSpec, first: Res) -> R<Flow> {
    let t = decode(f.sub & 0x7F);
    let Some(sel) = first.cards().first().copied() else { return Ok(Flow::Next) };
    let pokemons = g.st.slot_pokemons(t.p as usize, t.s);
    let idx = pokemons.iter().position(|c| *c == sel);
    if idx == Some(0) {
        crate::bail!("INVALID_PROMPT_RESULT");
    }
    let dest = zone_ref(f, d.destination);
    if let Some(i) = idx {
        // The Devolve event (Strange Timepiece: not an attack's effect, nothing prevents it).
        crate::engine::enter::devolve(g, t, pokemons.len() - i, dest, f.cause)?;
    }
    Ok(Flow::Next)
}

fn swap_bottom_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SwapPokemonCardSpec) -> R<Flow> {
    let Some(t) = slot_of(g, me, f, s.slot) else { return Ok(Flow::Next) };
    let Some(chosen) = reg_list(g, f, s.cards).first().copied() else { return Ok(Flow::Next) };
    let (tp, ts) = (t.p as usize, t.s);
    let old = if g.st.slot_pokemon(tp, ts).is_some() { g.st.slot(tp, ts).cards.get(0) } else { None };
    if g.st.locate(chosen).is_none() {
        return Ok(Flow::Next);
    }
    match old {
        // The new card goes onto the slot first and the old one leaves after, so the slot is never empty
        // (that would discard its attachments and reset it); the new card takes the old card's place at the
        // bottom of the stack. The same Pokémon (id2372): the Swap event.
        Some(old) => {
            crate::engine::enter::swap(g, t, old, chosen, zone_ref(f, s.into), crate::engine::enter::SwapPlace::Bottom, me, f.cause)?;
        }
        // No Pokémon there: the card is put onto the empty spot.
        None => {
            crate::engine::enter::enter_play(g, chosen, t, super::super::event::EnterMode::Effect, f.cause)?;
        }
    }
    Ok(Flow::Next)
}


#[cfg(test)]
mod effect_evolve_tests {
    //! An effect that evolves from the deck (Grand Tree, Salvatore) offers only the cards the Evolve event's checks
    //! allow: Palafin ex's Hero's Spirit locks every EnterPlay and Evolve of the card (docs/rulings/RULES.md
    //! "Evolution timing"), so Grand Tree doesn't offer it.
    use super::*;
    use crate::spec::event::RulesZone;
    use serde_json::json;

    #[test]
    fn grand_tree_does_not_offer_palafin_ex() {
        let mut names: Vec<&str> = vec!["Finizen TWM 59"; 4];
        names.extend(["Palafin TWM 60"; 4]);
        names.extend(["Palafin ex TWM 61"; 4]);
        names.extend(["Grand Tree SCR 136"; 1]);
        names.extend(["Water Energy MEE 3"; 47]);
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &json!({"me": {"reset": true, "hand_to_deck": true, "active": "Finizen TWM 59", "stadium": "Grand Tree SCR 136"}, "opp": {"reset": true, "active": "Finizen TWM 59"}})).unwrap();
        // Not the first turn: Grand Tree's limits don't apply.
        g.st.turn = g.st.turn.max(3);
        let me = g.st.active_player as usize;
        let t = SlotRef::new(me, g.st.players[me].active);
        let stadium = g.st.stadium_card().unwrap();
        let cause = crate::cause::Cause::of_trainer(&g, stadium, me as u8);
        let card = |g: &Game, name: &str| {
            let def = crate::carddb::def_by_full_name(name).unwrap();
            g.st.players[me].deck.iter().find(|c| g.st.cards[*c as usize].def == def).unwrap()
        };
        let (palafin, palafin_ex) = (card(&g, "Palafin TWM 60"), card(&g, "Palafin ex TWM 61"));
        assert!(effect_can_evolve(&mut g, t, palafin, RulesZone::Deck, cause).unwrap());
        assert!(!effect_can_evolve(&mut g, t, palafin_ex, RulesZone::Deck, cause).unwrap(), "a locked card isn't offered");
        // The refused event doesn't happen, so "if that Pokémon was evolved in this way" is false.
        assert!(!evolve_with(&mut g, t, palafin_ex, cause).unwrap());
        assert_eq!(g.st.slot_pokemon(me, t.s).map(|c| g.st.cdef(c).name), Some("Finizen"));
        assert!(evolve_with(&mut g, t, palafin, cause).unwrap());
    }
}
