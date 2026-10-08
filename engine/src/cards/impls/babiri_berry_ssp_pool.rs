//! Babiri Berry (SSP 163, Pokémon Tool): like Payapa Berry but against
//! attacks from your opponent's [M] Pokémon (60 less damage, discard this
//! card).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "BabiriBerrySSPPool",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 60, subject: SlotPred::Holder, source: SlotPred::TypeIs(ct::METAL), then_discard: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
