//! Values: players, zones, slots, numbers, conditions and card predicates
//! (vocabulary v1 "Selectors, predicates and values", "Conditions").
//!
//! Evaluation reads the game and never changes it.

use super::run::Frame;
use crate::effects::SlotRef;
use crate::game::Game;
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
    // Appended by F-passive.
    /// Pokémon of the player matching a (pure) slot predicate.
    SlotCount(Who, SlotPred),
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
    // Appended by F-passive.
    /// The player's marker `name` is set (by this card for `MarkerFrom::This`).
    HasMarker { who: Who, name: &'static str, from: super::ops::state::MarkerFrom },
    /// One of the player's Pokémon matches the slot predicate (pure predicates only).
    AnySlot(Who, SlotPred),
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
    // Appended by F-passive.
    /// The card prints that it provides the type of Energy.
    ProvidesType(CardType),
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
        Num::SlotCount(w, sp) => {
            let p = f.who(*w);
            g.st.players[p].in_play().iter().filter(|s| slot_pred_pure(g, me, SlotRef::new(p, **s), sp)).count() as i32
        }
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
        Cond::HasMarker { who, name, from } => super::ops::state::has_marker(g, me, f.who(*who), name, *from),
        Cond::AnySlot(w, sp) => {
            let p = f.who(*w);
            g.st.players[p].in_play().iter().any(|s| slot_pred_pure(g, me, SlotRef::new(p, *s), sp))
        }
        Cond::ZoneIs(z, n) => g.lst(zone_ref(f, *z)).len() as i32 == *n,
        Cond::LastCardInHand(w) => g.st.players[f.who(*w)].hand.iter().all(|c| c == me),
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
        Pred::ProvidesType(t) => d.provides.contains(t),
    }
}

// ---------------------------------------------------------------------------
// Slot predicates (appended by F-passive)

/// A predicate on a Pokémon in play (and what is attached to it).
pub enum SlotPred {
    Any,
    /// The slot this card is part of (the Pokémon it is, or the one it is attached to).
    Holder,
    Not(&'static SlotPred),
    All(&'static [SlotPred]),
    OneOf(&'static [SlotPred]),
    /// The top Pokémon carries the tag.
    Tag(u32),
    /// Some card of the slot has a Rule Box.
    RuleBox,
    /// The Pokémon's current type includes the type (effects applied).
    TypeIs(CardType),
    /// The Pokémon's printed type includes the type.
    PrintedTypeIs(CardType),
    /// The Pokémon's Energy provides the type (any-type Energy counts).
    Provides(CardType),
    /// The top card is a Basic Pokémon.
    Basic,
    /// The top card is of this Stage.
    StageIs(Stage),
    /// The top Pokémon evolves from another (an empty slot counts as one).
    Evolution,
    /// The Pokémon has an Ability after effects.
    HasAbility,
    /// The Pokémon prints a power of any kind.
    PrintsPower,
    IsActive,
    IsBench,
    /// The top Pokémon is this card.
    IsThisPokemon,
    /// The top Pokémon has this name.
    Named(&'static str),
    /// The Pokémon's Energy provides nothing.
    NoEnergyProvided,
    /// The Pokémon's remaining HP (with effects) is at most this much.
    RemainingHpAtMost(i32),
}

fn holds(g: &Game, me: CardId, s: SlotRef) -> bool {
    let sl = g.st.slot(s.p as usize, s.s);
    sl.cards.contains(me) || sl.tools.contains(me)
}

/// Evaluate the predicate when it needs no effect run (`None` otherwise).
fn slot_pred_ref(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> Option<bool> {
    let (p, sid) = (s.p as usize, s.s);
    let top = g.st.slot_pokemon(p, sid);
    Some(match sp {
        SlotPred::Any => true,
        SlotPred::Holder => holds(g, me, s),
        SlotPred::Not(x) => !slot_pred_ref(g, me, s, x)?,
        SlotPred::All(xs) => {
            for x in xs.iter() {
                if !slot_pred_ref(g, me, s, x)? {
                    return Some(false);
                }
            }
            true
        }
        SlotPred::OneOf(xs) => {
            for x in xs.iter() {
                if slot_pred_ref(g, me, s, x)? {
                    return Some(true);
                }
            }
            false
        }
        SlotPred::Tag(t) => top.map(|c| g.st.cdef(c).has_tag(*t)).unwrap_or(false),
        SlotPred::RuleBox => g.st.slot(p, sid).cards.iter().any(|c| g.st.cdef(c).has_rule_box()),
        SlotPred::PrintedTypeIs(t) => top.map(|c| g.st.cdef(c).card_type.contains(t)).unwrap_or(false),
        SlotPred::Basic => top.map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false),
        SlotPred::StageIs(st) => top.map(|c| g.st.cdef(c).stage == *st as u8).unwrap_or(false),
        SlotPred::Evolution => top.map(|c| !g.st.cdef(c).evolves_from.is_empty()).unwrap_or(true),
        SlotPred::PrintsPower => top.map(|c| !g.st.cdef(c).powers.is_empty()).unwrap_or(false),
        SlotPred::IsActive => g.st.players[p].active == sid,
        SlotPred::IsBench => g.st.players[p].active != sid,
        SlotPred::IsThisPokemon => top == Some(me),
        SlotPred::Named(n) => top.map(|c| g.st.cdef(c).name == *n).unwrap_or(false),
        SlotPred::TypeIs(_) | SlotPred::Provides(_) | SlotPred::HasAbility | SlotPred::NoEnergyProvided | SlotPred::RemainingHpAtMost(_) => return None,
    })
}

/// A predicate that needs no effect run; the effect-reading ones are false.
pub fn slot_pred_pure(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> bool {
    slot_pred_ref(g, me, s, sp).unwrap_or(false)
}

/// Evaluate a slot predicate, running the effects it reads (current types,
/// provided Energy, Abilities after effects).
pub fn slot_pred(g: &mut Game, me: CardId, s: SlotRef, sp: &SlotPred) -> crate::game::R<bool> {
    use crate::effects::Effect;
    if let Some(v) = slot_pred_ref(g, me, s, sp) {
        return Ok(v);
    }
    Ok(match sp {
        SlotPred::Not(x) => !slot_pred(g, me, s, x)?,
        SlotPred::All(xs) => {
            for x in xs.iter() {
                if !slot_pred(g, me, s, x)? {
                    return Ok(false);
                }
            }
            true
        }
        SlotPred::OneOf(xs) => {
            for x in xs.iter() {
                if slot_pred(g, me, s, x)? {
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
            matches!(e, Effect::CheckProvidedEnergy { energy_map, .. } if energy_map.iter().any(|m| m.provides.contains(t) || m.provides.contains(&crate::types::ct::ANY)))
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
        _ => false,
    })
}
