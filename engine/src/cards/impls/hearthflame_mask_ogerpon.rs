//! Hearthflame Mask Ogerpon (DRI 44): Fire Kagura - search your deck for a Basic [R] Energy card and attach it to 1 of your
//! Pokémon. Then, shuffle your deck. Searing Flame - 80; your opponent's Active Pokémon is now Burned.
//!
//! Fire Kagura: with a card in the deck, one search of 0 or 1 Basic [R] Energy attached to one Pokémon of yours (the Active
//! counts), then the shuffle (APR E-19; Flareon ex PRE 14). Searing Flame: a GainCondition(Burned) after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HearthflameMaskOgerpon",
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
                                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]),
                                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                                ..PickSpec::DEFAULT
                            },
                            destination: SearchDestination::AttachToPicked,
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
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
