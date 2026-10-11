//! Hoothoot (TEF 126): Silent Wing: 20; your opponent reveals their hand.
//!
//! The opponent shows their hand to the attacker after the damage (a Reveal routine; no card reacts to it), even when the
//! hand is empty ("reveals their hand").
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Hoothoot@TEF",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Reveal(RevealSpec {
                cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)),
                by: Who::Opp,
                to: Who::Me,
                when_empty: true,
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
