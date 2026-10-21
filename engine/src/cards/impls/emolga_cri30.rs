//! Emolga (CRI 30): Minor Errand-Running - search your deck for up to 2 Basic Energy cards, reveal them, and put them into
//! your hand. Then, shuffle your deck. Sky Return - 30; put this Pokémon and all attached cards into your hand.
//!
//! The search is Gimmighoul SSP 97's; Sky Return is one LeavePlay event (destination the hand) by the attack's effect, after
//! the damage (Meowth ex POR 62's Tuck Tail).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EmolgaCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::BasicEnergy,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Hand) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
