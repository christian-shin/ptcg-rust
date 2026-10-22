//! Salazzle ex (POR 16): Nasty Plot - search your deck for up to 2 cards and put them into your hand, then shuffle your deck. Dire Nails - 100; your opponent's Active Pokémon is now Burned and Poisoned. Switch this Pokémon with 1 of your Benched Pokémon.
//!
//! The search takes at least 1 card when the deck has one (id347) and at most 2; the shuffle follows (APR E-19; Cassiopeia's Search). Both conditions are inflicted together, then the self switch (Gliscor PFL 50, Cascoon ASC 14).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SalazzleEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Search(SearchSpec {
                    pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: false },
                    cancel: false,
                })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned, SpecialCondition::Poisoned])),
                Step::after_damage(switch_self()),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
