//! Teal Mask Ogerpon (TWM 24): Mountain Stroll — search your deck for up to 2
//! Basic Energy cards, reveal them and put them into your hand, then shuffle.
//! Ogre Comeback — 20+; 20 more damage for each of your opponent's Benched
//! Pokémon.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with a { superType: ENERGY,
//! energyType: BASIC } filter (shown to the opponent), min 0, max 2, no cancel.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TealMaskOgerponTWMPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        msg: "",
                        cancel: false,
                        shuffle_first: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                ],
                no: &[],
            }))],
        },
        // Ogre Comeback: 20 more damage for each of the opponent's Benched Pokémon.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::BenchCount(Who::Opp), &Num::Lit(20)), when: Cond::True }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
