//! Mawile (POR 32): Double Eater - 60x; discard up to 2 Energy cards from your hand, and this attack does 60 damage for each card
//! you discarded in this way.
//!
//! Chosen in the hand at step D (0 to 2 Energy cards, Basic or Special), the damage is 60 for each (0 with none) and the cards
//! are discarded from the hand (Skarmory PBL's Steel Cutter, any Energy).
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "MawilePOR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: HAND,
                predicate: Pred::Energy,
                bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(2), &Num::CardCount(HAND, Pred::Energy)) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), by: ChoiceDamageBy::Each(DamageOp::Set, 60) })),
            Step::after_damage(Op::Discard(DiscardSpec { from: HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
