//! Skarmory (PBL 60): Steel Cutter - 40x; discard up to 2 Basic [M] Energy cards from your hand, and this attack does 40 damage
//! for each card you discarded in this way.
//!
//! Chosen in the hand at step D (0 to 2 of the Basic [M] Energy cards there), the damage is 40 for each (0 with none) and the cards
//! are discarded from the hand (Team Rocket's Honchkrow's Rocket Feathers).
use crate::spec::prelude::*;

const BASIC_M: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")]);
const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "SkarmoryPBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: HAND,
                predicate: BASIC_M,
                bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(2), &Num::CardCount(HAND, BASIC_M)) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), by: ChoiceDamageBy::Each(DamageOp::Set, 40) })),
            Step::after_damage(Op::Discard(DiscardSpec { from: HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
