//! Landorus (POR 41): Rock Tumble - 50; this attack's damage isn't affected by Resistance. Screw Knuckle - 120; put an Energy attached to this Pokémon into your hand.
//!
//! Rock Tumble sets the `NoResistance` flag (Cynthia's Gible DRI 102). Screw Knuckle puts exactly 1 attached Energy card into the hand after the damage (Chien-Pao SSP 56).
use crate::spec::prelude::*;

const ATTACHED: ZoneRef = ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "Landorus",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                from: ATTACHED,
                predicate: Pred::Energy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::PutIntoHand(PutIntoHandSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
