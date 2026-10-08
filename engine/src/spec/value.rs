//! Values: players, zones, slots, numbers, conditions and card predicates
//! (vocabulary v1 "Selectors, predicates and values", "Conditions").
//!
//! Evaluation reads the game and never changes it.

use super::run::Frame;
use crate::effects::{Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    /// The player whose card is resolving.
    Me,
    Opp,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Zone {
    Deck,
    Hand,
    Discard,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ZoneRef(pub Who, pub Zone);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotExpr {
    /// The Pokémon this card is (or is attached to).
    This,
    Active(Who),
}

pub enum Num {
    Lit(i32),
    ZoneSize(ZoneRef),
    /// Cards in the zone matching the predicate.
    CardCount(ZoneRef, Pred),
    BenchCount(Who),
    OpenBench(Who),
    PrizesLeft(Who),
    DamageOn(SlotExpr),
    Turn,
    Add(&'static Num, &'static Num),
    Sub(&'static Num, &'static Num),
    Mul(&'static Num, &'static Num),
    Min(&'static Num, &'static Num),
    Max(&'static Num, &'static Num),
    /// `if cond { a } else { b }`.
    If(&'static Cond, &'static Num, &'static Num),
    /// Cards with this name in the player's discard pile and on their board.
    KnownCopies(Who, &'static str),
    // --- F-board appends ---
    /// Pokémon selected by the selector that satisfy the slot predicate.
    SlotCount(SlotSel, SlotPred),
    /// Energy on the selected Pokémon (summed). Needs a checked read: only ops
    /// that evaluate with `num_m` can use it.
    EnergyOn(SlotSel, EnergyUnit),
    /// Length of the printed cost of the attack being used.
    PrintedCost,
    // --- F-passive appends ---
    /// Prize cards the player has taken.
    PrizesTaken(Who),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

pub enum Cond {
    True,
    False,
    Not(&'static Cond),
    All(&'static [Cond]),
    Any(&'static [Cond]),
    Cmp(Num, CmpOp, Num),
    /// The zone holds a card matching the predicate.
    Nonempty(ZoneRef, Pred),
    BenchSpace(Who),
    /// The Pokémon is its player's Active Pokémon.
    IsActive(SlotExpr),
    // --- F-board appends ---
    /// The Pokémon satisfies the slot predicate.
    Slot(SlotExpr, SlotPred),
    /// Some selected Pokémon satisfies the predicate.
    AnySlot(SlotSel, SlotPred),
    /// There is at least one selected Pokémon, and every one satisfies the predicate.
    AllSlots(SlotSel, SlotPred),
    /// Adding these Special Conditions would change the Pokémon.
    WouldChangeConditions(SlotExpr, &'static [SpecialCondition]),
    /// A printed type (of a top Pokémon card) is held by a Pokémon in each selection.
    TypesShared(SlotSel, SlotSel),
    // --- F-passive appends ---
    /// The player's marker `name` is set (by this card for `MarkerFrom::This`).
    HasMarker { who: Who, name: &'static str, from: super::ops::state::MarkerFrom },
    /// Exactly `n` cards are in the zone.
    ZoneIs(ZoneRef, i32),
    /// Every card in the player's hand is this card (or the hand is empty).
    LastCardInHand(Who),
}

/// A card predicate.
pub enum Pred {
    Any,
    False,
    Not(&'static Pred),
    All(&'static [Pred]),
    OneOf(&'static [Pred]),
    Pokemon,
    Basic,
    Energy,
    BasicEnergy,
    Trainer,
    Item,
    Supporter,
    Tool,
    Stadium,
    Name(&'static str),
    HpAtMost(i32),
    /// The card has this card tag (`types::tag`).
    Tag(u32),
    /// A Pokémon with at least one attack.
    HasAttacks,
    // --- F-board appends ---
    Stage(u8),
    NameContains(&'static str),
    HasAbilityNamed(&'static str),
    HasAttackNamed(&'static str),
    // --- F-passive appends ---
    /// The card prints that it provides the type of Energy.
    ProvidesType(CardType),
    /// The Pokémon card is of this Stage.
    StageIs(Stage),
    /// The Pokémon's printed type includes the type.
    PrintedType(CardType),
}

impl Frame {
    pub fn who(&self, w: Who) -> usize {
        match w {
            Who::Me => self.p as usize,
            Who::Opp => 1 - self.p as usize,
        }
    }
}

pub fn zone_ref(f: &Frame, z: ZoneRef) -> ListRef {
    let p = f.who(z.0) as u8;
    match z.1 {
        Zone::Deck => ListRef::Deck(p),
        Zone::Hand => ListRef::Hand(p),
        Zone::Discard => ListRef::Discard(p),
    }
}

pub fn slot_of(g: &Game, me: CardId, f: &Frame, s: SlotExpr) -> Option<SlotRef> {
    match s {
        SlotExpr::Active(w) => {
            let p = f.who(w);
            Some(SlotRef::new(p, g.st.players[p].active))
        }
        SlotExpr::This => {
            for p in 0..2 {
                let pl = &g.st.players[p];
                let mut slots: SVec<SlotId, 9> = SVec::new();
                slots.push(pl.active);
                for b in pl.bench.iter() {
                    slots.push(*b);
                }
                for s in slots.iter().copied() {
                    let sl = &pl.slots[s as usize];
                    if sl.cards.contains(me) || sl.tools.contains(me) {
                        return Some(SlotRef::new(p, s));
                    }
                }
            }
            None
        }
    }
}

pub fn num(g: &Game, me: CardId, f: &Frame, n: &Num) -> i32 {
    match n {
        Num::Lit(v) => *v,
        Num::ZoneSize(z) => g.lst(zone_ref(f, *z)).len() as i32,
        Num::CardCount(z, p) => g.lst(zone_ref(f, *z)).iter().filter(|c| pred(g, **c, p)).count() as i32,
        Num::BenchCount(w) => {
            let pl = &g.st.players[f.who(*w)];
            pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32
        }
        Num::OpenBench(w) => empty_bench_slots(g, f.who(*w)).len() as i32,
        Num::PrizesLeft(w) => g.st.players[f.who(*w)].prize_left() as i32,
        Num::DamageOn(s) => slot_of(g, me, f, *s).map(|s| g.st.slot(s.p as usize, s.s).damage).unwrap_or(0),
        Num::Turn => g.st.turn as i32,
        Num::Add(a, b) => num(g, me, f, a) + num(g, me, f, b),
        Num::Sub(a, b) => num(g, me, f, a) - num(g, me, f, b),
        Num::Mul(a, b) => num(g, me, f, a) * num(g, me, f, b),
        Num::Min(a, b) => num(g, me, f, a).min(num(g, me, f, b)),
        Num::Max(a, b) => num(g, me, f, a).max(num(g, me, f, b)),
        Num::If(c, a, b) => {
            if cond(g, me, f, c) {
                num(g, me, f, a)
            } else {
                num(g, me, f, b)
            }
        }
        Num::KnownCopies(w, name) => {
            let p = f.who(*w);
            let mut n = g.st.players[p].discard.iter().filter(|c| g.st.cdef(*c).name == *name).count();
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
                n += g.st.slot(p, *s).cards.iter().filter(|c| g.st.cdef(*c).name == *name).count();
            }
            n as i32
        }
        Num::SlotCount(sel, sp) => slots_of(g, me, f, sel).iter().filter(|s| slot_pred(g, me, **s, sp).expect("checked read: evaluate with num_m")).count() as i32,
        Num::EnergyOn(..) => panic!("Num::EnergyOn needs a checked read (num_m)"),
        Num::PrintedCost => printed_cost(g, f),
        Num::PrizesTaken(w) => 6 - g.st.players[f.who(*w)].prize_left() as i32,
    }
}

pub fn cond(g: &Game, me: CardId, f: &Frame, c: &Cond) -> bool {
    match c {
        Cond::True => true,
        Cond::False => false,
        Cond::Not(c) => !cond(g, me, f, c),
        Cond::All(cs) => cs.iter().all(|c| cond(g, me, f, c)),
        Cond::Any(cs) => cs.iter().any(|c| cond(g, me, f, c)),
        Cond::Cmp(a, op, b) => {
            let (a, b) = (num(g, me, f, a), num(g, me, f, b));
            match op {
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Ge => a >= b,
                CmpOp::Gt => a > b,
            }
        }
        Cond::Nonempty(z, p) => g.lst(zone_ref(f, *z)).iter().any(|c| pred(g, *c, p)),
        Cond::BenchSpace(w) => !empty_bench_slots(g, f.who(*w)).is_empty(),
        Cond::IsActive(s) => slot_of(g, me, f, *s).map(|s| g.st.players[s.p as usize].active == s.s).unwrap_or(false),
        Cond::Slot(e, sp) => match slot_of(g, me, f, *e) {
            Some(s) => slot_pred(g, me, s, sp).expect("checked read: evaluate with cond_m"),
            None => false,
        },
        Cond::AnySlot(sel, sp) => slots_of(g, me, f, sel).iter().any(|s| slot_pred(g, me, *s, sp).expect("checked read: evaluate with cond_m")),
        Cond::AllSlots(sel, sp) => {
            let v = slots_of(g, me, f, sel);
            !v.is_empty() && v.iter().all(|s| slot_pred(g, me, *s, sp).expect("checked read: evaluate with cond_m"))
        }
        Cond::WouldChangeConditions(e, cs) => match slot_of(g, me, f, *e) {
            Some(s) => crate::engine::phase::would_change_special_conditions(g.st.slot(s.p as usize, s.s), cs),
            None => false,
        },
        Cond::HasMarker { who, name, from } => super::ops::state::has_marker(g, me, f.who(*who), name, *from),
        Cond::ZoneIs(z, n) => g.lst(zone_ref(f, *z)).len() as i32 == *n,
        Cond::LastCardInHand(w) => g.st.players[f.who(*w)].hand.iter().all(|c| c == me),
        Cond::TypesShared(a, b) => {
            let printed = |sel: &SlotSel| -> Vec<CardType> {
                let mut out: Vec<CardType> = Vec::new();
                for s in slots_of(g, me, f, sel).iter() {
                    if let Some(c) = g.st.slot_pokemon(s.p as usize, s.s) {
                        for t in g.st.cdef(c).card_type {
                            if !out.contains(t) {
                                out.push(*t);
                            }
                        }
                    }
                }
                out
            };
            let (x, y) = (printed(a), printed(b));
            x.iter().any(|t| y.contains(t))
        }
    }
}

pub fn pred(g: &Game, c: CardId, p: &Pred) -> bool {
    let d = g.st.cdef(c);
    match p {
        Pred::Any => true,
        Pred::False => false,
        Pred::Not(p) => !pred(g, c, p),
        Pred::All(ps) => ps.iter().all(|p| pred(g, c, p)),
        Pred::OneOf(ps) => ps.iter().any(|p| pred(g, c, p)),
        Pred::Pokemon => d.is_pokemon(),
        Pred::Basic => d.is_pokemon() && d.stage == Stage::Basic as u8,
        Pred::Energy => d.is_energy(),
        Pred::BasicEnergy => d.is_energy() && d.energy_type == EnergyType::Basic as u8,
        Pred::Trainer => d.is_trainer(),
        Pred::Item => d.is_trainer() && d.trainer_type == TrainerType::Item as u8,
        Pred::Supporter => d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8,
        Pred::Tool => d.is_trainer() && d.trainer_type == TrainerType::Tool as u8,
        Pred::Stadium => d.is_trainer() && d.trainer_type == TrainerType::Stadium as u8,
        Pred::Name(n) => d.name == *n,
        Pred::HpAtMost(n) => d.is_pokemon() && d.hp <= *n,
        Pred::Tag(t) => d.has_tag(*t),
        Pred::HasAttacks => d.is_pokemon() && !d.attacks.is_empty(),
        Pred::Stage(st) => d.is_pokemon() && d.stage == *st,
        Pred::NameContains(n) => d.name.contains(*n),
        Pred::HasAbilityNamed(n) => d.is_pokemon() && d.powers.iter().any(|p| p.power_type == PowerType::Ability as u8 && p.name == *n),
        Pred::HasAttackNamed(n) => d.attacks.iter().any(|a| a.name == *n),
        Pred::ProvidesType(t) => d.provides.contains(t),
        Pred::StageIs(st) => d.is_pokemon() && d.stage == *st as u8,
        Pred::PrintedType(t) => d.is_pokemon() && d.card_type.contains(t),
    }
}

// ---------------------------------------------------------------------------
// F-board appends: slot selectors, slot predicates, checked reads

/// A collection of Pokémon in play.
pub enum SlotSel {
    One(SlotExpr),
    /// The occupied Bench slots of a player.
    Bench(Who),
    /// Every Pokémon of a player: the Active, then the Bench.
    Pokemon(Who),
    Filtered(&'static SlotSel, SlotPred),
    /// Every Pokémon of a player; a prompt lists the Bench before the Active Spot.
    PokemonBenchFirst(Who),
}

/// A predicate on a Pokémon in play.
pub enum SlotPred {
    Any,
    Not(&'static SlotPred),
    All(&'static [SlotPred]),
    OneOf(&'static [SlotPred]),
    /// Has at least one damage counter.
    Damaged,
    IsActive,
    IsBench,
    /// The top Pokémon card satisfies the card predicate.
    Top(Pred),
    /// Is affected by any Special Condition.
    HasCondition,
    /// The type of the Pokémon as the game checks it (Energy and Stadiums can
    /// change it): a checked read.
    TypeIs(CardType),
    /// The printed type of the top Pokémon card.
    PrintedTypeIs(CardType),
    /// The Pokémon card directly under this card in the stack has this name.
    CardBelowThis(&'static str),
    /// The Pokémon was played (or evolved) this turn.
    PlayedThisTurn,
    // --- F-passive appends ---
    /// The slot this card is part of (the Pokémon it is, or the one it is attached to).
    Holder,
    /// The top Pokémon carries the tag.
    Tag(u32),
    /// Some card of the slot has a Rule Box.
    RuleBox,
    /// The Pokémon's Energy provides the type (any-type Energy counts): a checked read.
    Provides(CardType),
    /// The top card is a Basic Pokémon.
    Basic,
    /// The top card is of this Stage.
    StageIs(Stage),
    /// The top Pokémon evolves from another (an empty slot counts as one).
    Evolution,
    /// The Pokémon has an Ability after effects: a checked read.
    HasAbility,
    /// The Pokémon prints a power of any kind.
    PrintsPower,
    /// The top Pokémon is this card.
    IsThisPokemon,
    /// The top Pokémon has this name.
    Named(&'static str),
    /// The Pokémon's Energy provides nothing: a checked read.
    NoEnergyProvided,
    /// Some card of the slot carries the tag.
    AnyCardTag(u32),
    /// The slot has an Energy card attached.
    HasEnergy,
    /// The Pokémon's remaining HP (with effects) is at most this much: a checked read.
    RemainingHpAtMost(i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergyUnit {
    /// Attached Special Energy cards. Pure.
    SpecialEnergyCards,
    /// Provided Energy: the `provides` entries equal to the type or to ANY
    /// (CheckProvidedEnergy).
    Provided(CardType),
    /// Every `provides` entry of every Energy card (CheckProvidedEnergy).
    ProvidedUnits,
}

/// Printed cost length of the attack being used.
fn printed_cost(g: &Game, f: &Frame) -> i32 {
    match crate::prefabs::attack_data(g, f.eff) {
        Some((_, _, a, _)) => crate::engine::attack::attack_def(g, a).cost.len() as i32,
        None => 0,
    }
}

pub fn slots_of(g: &Game, me: CardId, f: &Frame, sel: &SlotSel) -> SVec<SlotRef, 9> {
    let mut out: SVec<SlotRef, 9> = SVec::new();
    match sel {
        SlotSel::One(e) => {
            if let Some(s) = slot_of(g, me, f, *e) {
                if !g.st.slot(s.p as usize, s.s).cards.is_empty() {
                    out.push(s);
                }
            }
        }
        SlotSel::Bench(w) => {
            let p = f.who(*w);
            let pl = &g.st.players[p];
            for b in pl.bench.iter() {
                if !pl.slots[*b as usize].cards.is_empty() {
                    out.push(SlotRef::new(p, *b));
                }
            }
        }
        SlotSel::Pokemon(w) | SlotSel::PokemonBenchFirst(w) => {
            let p = f.who(*w);
            for s in g.st.players[p].in_play().iter() {
                out.push(SlotRef::new(p, *s));
            }
        }
        SlotSel::Filtered(inner, sp) => {
            for s in slots_of(g, me, f, inner).iter() {
                if slot_pred(g, me, *s, sp).unwrap_or(true) {
                    out.push(*s);
                }
            }
        }
    }
    out
}

/// The pure reading of a slot predicate; `None` when it needs a checked read.
pub fn slot_pred(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> Option<bool> {
    let (p, id) = (s.p as usize, s.s);
    let slot = g.st.slot(p, id);
    Some(match sp {
        SlotPred::Any => true,
        SlotPred::Not(q) => !slot_pred(g, me, s, q)?,
        SlotPred::All(qs) => {
            let mut r = true;
            for q in qs.iter() {
                r &= slot_pred(g, me, s, q)?;
            }
            r
        }
        SlotPred::OneOf(qs) => {
            let mut r = false;
            for q in qs.iter() {
                r |= slot_pred(g, me, s, q)?;
            }
            r
        }
        SlotPred::Damaged => slot.damage > 0,
        SlotPred::IsActive => g.st.players[p].active == id,
        SlotPred::IsBench => g.st.players[p].active != id,
        SlotPred::Top(q) => g.st.slot_pokemon(p, id).map(|c| pred(g, c, q)).unwrap_or(false),
        SlotPred::HasCondition => !slot.special_conditions.is_empty(),
        SlotPred::TypeIs(_) => return None,
        SlotPred::PrintedTypeIs(t) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).card_type.contains(t)).unwrap_or(false),
        SlotPred::CardBelowThis(name) => {
            let stack = g.st.slot_pokemons(p, id);
            stack.iter().position(|c| *c == me).and_then(|i| i.checked_sub(1)).map_or(false, |i| g.st.cdef(stack.as_slice()[i]).name == *name)
        }
        SlotPred::PlayedThisTurn => slot.pokemon_played_turn == g.st.turn as i32,
        SlotPred::Holder => slot.cards.contains(me) || slot.tools.contains(me),
        SlotPred::Tag(t) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).has_tag(*t)).unwrap_or(false),
        SlotPred::RuleBox => slot.cards.iter().any(|c| g.st.cdef(c).has_rule_box()),
        SlotPred::Basic => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false),
        SlotPred::StageIs(st) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).stage == *st as u8).unwrap_or(false),
        SlotPred::Evolution => g.st.slot_pokemon(p, id).map(|c| !g.st.cdef(c).evolves_from.is_empty()).unwrap_or(true),
        SlotPred::PrintsPower => g.st.slot_pokemon(p, id).map(|c| !g.st.cdef(c).powers.is_empty()).unwrap_or(false),
        SlotPred::IsThisPokemon => g.st.slot_pokemon(p, id) == Some(me),
        SlotPred::Named(n) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).name == *n).unwrap_or(false),
        SlotPred::AnyCardTag(t) => slot.cards.iter().any(|c| g.st.cdef(c).has_tag(*t)),
        SlotPred::HasEnergy => !slot.energies.is_empty(),
        SlotPred::Provides(_) | SlotPred::HasAbility | SlotPred::NoEnergyProvided | SlotPred::RemainingHpAtMost(_) => return None,
    })
}

/// Slot predicate with checked reads.
pub fn slot_pred_m(g: &mut Game, me: CardId, s: SlotRef, sp: &SlotPred) -> R<bool> {
    Ok(match sp {
        SlotPred::Not(q) => !slot_pred_m(g, me, s, q)?,
        SlotPred::All(qs) => {
            for q in qs.iter() {
                if !slot_pred_m(g, me, s, q)? {
                    return Ok(false);
                }
            }
            true
        }
        SlotPred::OneOf(qs) => {
            for q in qs.iter() {
                if slot_pred_m(g, me, s, q)? {
                    return Ok(true);
                }
            }
            false
        }
        SlotPred::TypeIs(t) => {
            let types = crate::engine::game_effect::pokemon_types(g, s);
            let (e, _) = g.run_fx(Effect::CheckPokemonType { target: s, card_types: types })?;
            matches!(e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(t))
        }
        SlotPred::Provides(t) => {
            let (e, _) = g.run_fx(Effect::CheckProvidedEnergy { p: s.p, source: s, energy_map: SVec::new() })?;
            matches!(e, Effect::CheckProvidedEnergy { energy_map, .. } if energy_map.iter().any(|m| m.provides.contains(t) || m.provides.contains(&ct::ANY)))
        }
        SlotPred::NoEnergyProvided => {
            let (e, _) = g.run_fx(Effect::CheckProvidedEnergy { p: s.p, source: s, energy_map: SVec::new() })?;
            matches!(e, Effect::CheckProvidedEnergy { energy_map, .. } if energy_map.is_empty())
        }
        SlotPred::RemainingHpAtMost(n) => {
            if g.st.slot_pokemon(s.p as usize, s.s).is_none() {
                return Ok(false);
            }
            let hp = crate::engine::check::check_hp(g, s.p as usize, s.s)?;
            hp - g.st.slot(s.p as usize, s.s).damage <= *n
        }
        SlotPred::HasAbility => {
            let Some(src) = g.st.slot_pokemon(s.p as usize, s.s) else { return Ok(false) };
            let mut powers = SVec::new();
            for i in 0..g.st.cdef(src).powers.len() {
                powers.push(crate::effects::PowerRef { card: src, index: i as u8 });
            }
            let (e, _) = g.run_fx(Effect::CheckPokemonPowers { p: s.p, target: src, powers })?;
            match e {
                Effect::CheckPokemonPowers { powers, .. } => powers.iter().any(|r| g.st.cdef(r.card).powers[r.index as usize].power_type == PowerType::Ability as u8),
                _ => false,
            }
        }
        _ => slot_pred(g, me, s, sp).expect("pure slot predicate"),
    })
}

/// The selected Pokémon, with checked reads in the filters.
pub fn slots_m(g: &mut Game, me: CardId, f: &Frame, sel: &SlotSel) -> R<SVec<SlotRef, 9>> {
    match sel {
        SlotSel::Filtered(inner, sp) => {
            let mut out: SVec<SlotRef, 9> = SVec::new();
            for s in slots_m(g, me, f, inner)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    out.push(*s);
                }
            }
            Ok(out)
        }
        _ => Ok(slots_of(g, me, f, sel)),
    }
}

/// `num` with checked reads (CheckProvidedEnergy, CheckPokemonType).
pub fn num_m(g: &mut Game, me: CardId, f: &Frame, n: &Num) -> R<i32> {
    Ok(match n {
        Num::Add(a, b) => num_m(g, me, f, a)? + num_m(g, me, f, b)?,
        Num::Sub(a, b) => num_m(g, me, f, a)? - num_m(g, me, f, b)?,
        Num::Mul(a, b) => num_m(g, me, f, a)? * num_m(g, me, f, b)?,
        Num::Min(a, b) => num_m(g, me, f, a)?.min(num_m(g, me, f, b)?),
        Num::Max(a, b) => num_m(g, me, f, a)?.max(num_m(g, me, f, b)?),
        Num::If(c, a, b) => {
            if cond_m(g, me, f, c)? {
                num_m(g, me, f, a)?
            } else {
                num_m(g, me, f, b)?
            }
        }
        Num::SlotCount(sel, sp) => {
            let mut n = 0;
            for s in slots_m(g, me, f, sel)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    n += 1;
                }
            }
            n
        }
        Num::EnergyOn(sel, unit) => {
            let mut total = 0;
            for s in slots_m(g, me, f, sel)?.iter() {
                total += match unit {
                    EnergyUnit::SpecialEnergyCards => g
                        .st
                        .slot(s.p as usize, s.s)
                        .cards
                        .iter()
                        .filter(|c| {
                            let d = g.st.cdef(*c);
                            d.is_energy() && d.energy_type == EnergyType::Special as u8
                        })
                        .count() as i32,
                    EnergyUnit::Provided(_) | EnergyUnit::ProvidedUnits => {
                        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: s.p, source: *s, energy_map: SVec::new() })?;
                        let mut k = 0;
                        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                            for em in energy_map.iter() {
                                k += match unit {
                                    EnergyUnit::Provided(t) => em.provides.iter().filter(|x| **x == *t || **x == ct::ANY).count() as i32,
                                    _ => em.provides.len() as i32,
                                };
                            }
                        }
                        k
                    }
                };
            }
            total
        }
        _ => num(g, me, f, n),
    })
}

/// `cond` with checked reads.
pub fn cond_m(g: &mut Game, me: CardId, f: &Frame, c: &Cond) -> R<bool> {
    Ok(match c {
        Cond::Not(c) => !cond_m(g, me, f, c)?,
        Cond::All(cs) => {
            for c in cs.iter() {
                if !cond_m(g, me, f, c)? {
                    return Ok(false);
                }
            }
            true
        }
        Cond::Any(cs) => {
            for c in cs.iter() {
                if cond_m(g, me, f, c)? {
                    return Ok(true);
                }
            }
            false
        }
        Cond::Cmp(a, op, b) => {
            let (a, b) = (num_m(g, me, f, a)?, num_m(g, me, f, b)?);
            match op {
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Ge => a >= b,
                CmpOp::Gt => a > b,
            }
        }
        Cond::Slot(e, sp) => match slot_of(g, me, f, *e) {
            Some(s) => slot_pred_m(g, me, s, sp)?,
            None => false,
        },
        Cond::AnySlot(sel, sp) => {
            for s in slots_m(g, me, f, sel)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    return Ok(true);
                }
            }
            false
        }
        Cond::AllSlots(sel, sp) => {
            let v = slots_m(g, me, f, sel)?;
            if v.is_empty() {
                return Ok(false);
            }
            for s in v.iter() {
                if !slot_pred_m(g, me, *s, sp)? {
                    return Ok(false);
                }
            }
            true
        }
        _ => cond(g, me, f, c),
    })
}


/// A slot predicate read without running effects: the checked ones are false.
pub fn slot_pred_pure(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> bool {
    slot_pred(g, me, s, sp).unwrap_or(false)
}
