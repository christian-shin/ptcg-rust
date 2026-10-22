//! Pidove (BLK 71): Scout - your opponent reveals their hand. Stampede - 10.
//!
//! The opponent shows their hand to the attacker (Espurr SSP 84's See Through).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PidoveBLK",
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
