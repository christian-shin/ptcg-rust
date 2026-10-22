//! Sudowoodo (CRI 43): Trials and Trip-ulations - search your deck for up to 2 Transformation Tome cards, reveal them, and put
//! them into your hand. Then, shuffle your deck. Rock Hurl - 30; this attack's damage isn't affected by Resistance.
//!
//! One Search of 0 to 2 cards named Transformation Tome revealed into the hand, one shuffle after it (APR E-19). Rock Hurl sets
//! the NoResistance attack flag before the damage (Cornerstone Mask Ogerpon ex's Demolish).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sudowoodo",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { predicate: Pred::Name("Transformation Tome"), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
