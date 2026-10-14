//! Ceruledge (PFL 20): Infernal Slash: 220; discard 4 Basic [R] Energy cards from your hand or this attack does nothing.
//!
//! With fewer than 4 Basic [R] Energy cards in hand nothing is discarded and the damage is set to 0 (id2158: the cards are
//! discarded in full or not at all); otherwise exactly 4 are chosen and discarded after the damage (one Discard event carrying
//! the 4 cards, this attack its cause). The discard is the attack's effect, not a cost (a cost is a Trainer's or an
//! Ability's, APR E-30).
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
                purpose: Some(PromptPurpose::DiscardCard),
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
