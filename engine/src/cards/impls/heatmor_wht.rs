//! Heatmor (WHT 19): Licking Catch - search your deck for up to 3 in any combination of [R] Pokémon and Basic [R] Energy cards, reveal them, and put them into your hand. Then, shuffle your deck. Fire Claws - 60.
//!
//! An attack's "up to 3" may be 0 (id2301). One PutIntoHand of the revealed cards, then one shuffle (APR E-19; Celebi MEG 12).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Heatmor",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::FIRE)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")])]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
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
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
