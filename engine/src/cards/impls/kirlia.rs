//! Kirlia (M1S / ASC): Call Sign - search your deck for up to 3 Pokémon and put them into your hand. Then, shuffle your deck.
//! Psyshot - 30.
//!
//! One Search op (nothing revealed: the text doesn't say reveal), then one shuffle after it (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kirlia@MEG|ASC",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                cancel: false,
            })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
