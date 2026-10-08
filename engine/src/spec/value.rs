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
    /// Card register `r` (a scratch list: looked-at cards, chosen cards).
    Scratch(u8),
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
    /// Cards in card register `r`.
    RegCount(u8),
    /// Pokémon in play (top card matching the predicate).
    InPlayCount(Who, PlayScope, Pred),
    /// Distinct first provided types among the cards of a zone matching the predicate.
    DistinctTypes(ZoneRef, Pred),
}

/// Which Pokémon in play a count or condition looks at.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayScope {
    All,
    Bench,
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
    /// Like `Nonempty`, not counting the resolving card.
    NonemptyOther(ZoneRef, Pred),
    /// Card register `r` holds cards.
    Chosen(u8),
    /// The player played an Ancient Supporter this turn.
    AncientSupporterPlayed(Who),
    /// At least `at_least` cards named `name` in the player's discard pile and in play.
    KnownCopies { who: Who, name: &'static str, at_least: i32 },
    /// A Pokémon in play whose top card matches.
    InPlay(Who, PlayScope, Pred),
    /// A Pokémon in play with any card of its stack matching.
    InPlayAny(Who, PlayScope, Pred),
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
    /// Card tag (`types::tag`).
    Tag(u32),
    /// Printed Pokémon type.
    PokemonType(u8),
    /// Energy card that provides the type.
    Provides(u8),
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
        Zone::Scratch(r) => ListRef::Temp(f.cards[r as usize]),
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
        Num::RegCount(r) => reg_list(g, f, *r).len() as i32,
        Num::InPlayCount(w, scope, p) => in_play(g, f.who(*w), *scope).iter().filter(|(_, top, _)| pred(g, *top, p)).count() as i32,
        Num::DistinctTypes(z, p) => {
            let mut types: Vec<u8> = Vec::new();
            for c in g.lst(zone_ref(f, *z)).iter() {
                let d = g.st.cdef(*c);
                if pred(g, *c, p) {
                    if let Some(t) = d.provides.first() {
                        if !types.contains(t) {
                            types.push(*t);
                        }
                    }
                }
            }
            types.len() as i32
        }
    }
}

/// The cards of card register `r` (empty while it is unset).
pub fn reg_list<'a>(g: &'a Game, f: &Frame, r: u8) -> &'a [CardId] {
    match f.cards[r as usize] {
        super::run::NONE => &[],
        i => g.lst(ListRef::Temp(i)),
    }
}

/// Pokémon in play, Active first: (slot, top card, stack).
pub fn in_play(g: &Game, p: usize, scope: PlayScope) -> Vec<(SlotId, CardId, Vec<CardId>)> {
    let pl = &g.st.players[p];
    let mut slots: Vec<SlotId> = Vec::new();
    if scope == PlayScope::All {
        slots.push(pl.active);
    }
    slots.extend(pl.bench.iter().copied());
    slots
        .into_iter()
        .filter_map(|s| g.st.slot_pokemon(p, s).map(|top| (s, top, pl.slots[s as usize].cards.iter().collect::<Vec<_>>())))
        .collect()
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
        Cond::NonemptyOther(z, p) => g.lst(zone_ref(f, *z)).iter().any(|c| *c != me && pred(g, *c, p)),
        Cond::Chosen(r) => !reg_list(g, f, *r).is_empty(),
        Cond::AncientSupporterPlayed(w) => g.st.players[f.who(*w)].ancient_supporter,
        Cond::KnownCopies { who, name, at_least } => {
            let p = f.who(*who);
            let mut n = g.lst(ListRef::Discard(p as u8)).iter().filter(|c| g.st.cdef(**c).name == *name).count() as i32;
            for (_, _, stack) in in_play(g, p, PlayScope::All) {
                n += stack.iter().filter(|c| g.st.cdef(**c).name == *name).count() as i32;
            }
            n >= *at_least
        }
        Cond::InPlay(w, scope, p) => in_play(g, f.who(*w), *scope).iter().any(|(_, top, _)| pred(g, *top, p)),
        Cond::InPlayAny(w, scope, p) => in_play(g, f.who(*w), *scope).iter().any(|(_, _, stack)| stack.iter().any(|c| pred(g, *c, p))),
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
        Pred::PokemonType(t) => d.is_pokemon() && d.card_type.contains(t),
        Pred::Provides(t) => d.is_energy() && d.provides.contains(t),
    }
}
