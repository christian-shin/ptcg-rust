//! Kirlia (MEG 59 / ASC 88): Call Sign - search your deck for up to 3 Pokémon, reveal them, and put them into your hand. Then, shuffle your deck.
//! Psyshot - 30.
//!
//! One Search op (revealed to the opponent: the text says "reveal them"), then one shuffle after it (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kirlia",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
