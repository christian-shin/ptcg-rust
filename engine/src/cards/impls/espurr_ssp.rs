//! Espurr (SSP 84): See Through - your opponent reveals their hand. Psyshot - 20.
//!
//! The opponent's hand is revealed to the attack's player (Hoothoot TEF 126's Silent Wing).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EspurrSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Reveal(RevealSpec {
                cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)),
                by: Who::Opp,
                to: Who::Me,
                when_empty: true,
            })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
