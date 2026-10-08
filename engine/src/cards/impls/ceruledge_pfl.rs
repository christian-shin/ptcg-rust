//! Ceruledge (M2 / PFL): Purgatory Slash — 220; discard 4 Basic [R] Energy
//! cards from your hand or this attack does nothing.
//!
//! Twinleaf: with fewer than 4 "Fire Energy" basics in hand the damage is set
//! to 0 at once; otherwise a non-cancellable ChooseCardsPrompt (exactly 4)
//! whose callback discards them (or zeroes the damage if the filtered result
//! is not exactly 4).
use crate::spec::prelude::*;

const FIRE_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]);
const HAS_FOUR: Cond = Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Hand), FIRE_ENERGY), CmpOp::Ge, Num::Lit(4));

pub static SPEC: CardSpec = CardSpec {
    class: "Ceruledge@PFL",
    // Purgatory Slash: discard 4 Basic [R] Energy cards from your hand or this attack does nothing.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::If(IfSpec { cond: Cond::Not(&HAS_FOUR), yes: &[Step::new(damage_is(Num::Lit(0)))], no: &[] })),
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: FIRE_ENERGY,
                bounds: Bounds { min: Num::If(&HAS_FOUR, &Num::Lit(4), &Num::Lit(0)), max: Num::If(&HAS_FOUR, &Num::Lit(4), &Num::Lit(0)) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
