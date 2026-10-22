//! Dipplin (DRI 17): Energy Loop — 50; put an Energy attached to this Pokémon into your hand.
//!
//! After the damage, one attached Energy card is chosen and put into the hand (a PutIntoHand event caused by the attack).
use crate::spec::prelude::*;

const ATTACHED: ZoneRef = ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "DipplinDRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec { from: ATTACHED, predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::PutIntoHand(PutIntoHandSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
