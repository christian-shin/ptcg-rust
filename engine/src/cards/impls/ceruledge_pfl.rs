//! Ceruledge (PFL 20): Infernal Slash: 220; discard 4 Basic [R] Energy cards from your hand. If you can't discard 4 cards in
//! this way, this attack does nothing.
//!
//! The 4 cards are chosen before the damage (step D) and the damage reads that choice (`ChoiceDamageBy::NothingUnless`):
//! with fewer than 4 Basic [R] Energy cards in hand none is chosen, nothing is discarded and the attack does no damage
//! (id2158: the cards are discarded in full or not at all); otherwise exactly 4 are chosen and discarded after the damage
//! (one Discard event carrying the 4 cards, this attack its cause). The discard is the attack's effect, not a cost (a cost
//! is a Trainer's or an Ability's, APR E-30).
use crate::spec::prelude::*;

const FIRE_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]);
const HAS_FOUR: Cond = Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Hand), FIRE_ENERGY), CmpOp::Ge, Num::Lit(4));

pub static SPEC: CardSpec = CardSpec {
    class: "CeruledgePFL",
    // Infernal Slash: discard 4 Basic [R] Energy cards from your hand. If you can't discard 4 cards in this way, this attack
    // does nothing.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: FIRE_ENERGY,
                bounds: Bounds { min: Num::If(&HAS_FOUR, &Num::Lit(4), &Num::Lit(0)), max: Num::If(&HAS_FOUR, &Num::Lit(4), &Num::Lit(0)) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), by: ChoiceDamageBy::NothingUnless(4) })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
