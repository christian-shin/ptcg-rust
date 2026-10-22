//! Wellspring Mask Ogerpon (DRI 67): Water Kagura - search your deck for a Basic [W] Energy card and attach it to 1 of your
//! Pokémon. Then, shuffle your deck. Bubble Drain - 100; heal 30 damage from this Pokémon.
//!
//! Water Kagura: with a card in the deck, one search of 0 or 1 Basic [W] Energy attached to one Pokémon of yours, then the
//! shuffle (APR E-19; Hearthflame Mask Ogerpon DRI 44). Bubble Drain heals 30 from this Pokémon after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WellspringMaskOgerpon",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::AttachToPicked,
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(heal_active(30))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
