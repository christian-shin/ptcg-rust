//! Hydrapple (DRI 18): Hydra Breath — discard 6 Basic [G] Energy cards from
//! your hand, and Knock Out your opponent's Active Pokémon. If you can't discard 6
//! cards in this way, this attack does nothing. Whip Smash — 140.
//!
//! Rule: with fewer than 6 Basic Grass Energy in hand nothing is discarded and
//! nothing happens (id2158); with 6 or more you choose exactly 6 (Discard), then
//! `Op::KnockOut` on the opponent's Active Pokémon: a Knock Out by an effect, resolved
//! at the next state check with every other Knock Out (D1, id2089). "Prevent all
//! effects of attacks" refuses it, and the discard has happened (id2427: the rest of
//! the effect goes on).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hydrapple",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])), CmpOp::Ge, Num::Lit(6)), yes: &[Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]), bounds: Bounds { min: Num::Lit(6), max: Num::Lit(6) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })), Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Active(Who::Opp), when: Cond::True }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
