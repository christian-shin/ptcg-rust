//! Lurantis (DRI 14): Petal Blade Dance - 130; discard 2 Basic [G] Energy cards from your hand. If you can't discard 2 cards in this way, this attack does nothing.
//!
//! The 2 cards are chosen before the damage and the damage reads that choice (`ChoiceDamageBy::NothingUnless`): with fewer than 2 Basic [G] Energy cards in hand none is chosen, nothing is
//! discarded and the attack does no damage (id2158); otherwise exactly 2 are chosen and discarded after the damage (Ceruledge PFL 20).
use crate::spec::prelude::*;

const GRASS_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]);
const HAS_TWO: Cond = Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Hand), GRASS_ENERGY), CmpOp::Ge, Num::Lit(2));

pub static SPEC: CardSpec = CardSpec {
    class: "Lurantis",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    predicate: GRASS_ENERGY,
                    bounds: Bounds { min: Num::If(&HAS_TWO, &Num::Lit(2), &Num::Lit(0)), max: Num::If(&HAS_TWO, &Num::Lit(2), &Num::Lit(0)) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), by: ChoiceDamageBy::NothingUnless(2) })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
