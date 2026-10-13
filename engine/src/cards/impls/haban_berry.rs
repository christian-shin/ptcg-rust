//! Haban Berry (PRE 111, Pokémon Tool): if the Pokémon this card is attached to is damaged by an attack from your opponent's [N]
//! Pokémon, it takes 60 less damage (after applying Weakness and Resistance), and discard this card.
//!
//! One `Modifier::DamageTaken` (60, subject the Holder, attacker `TypeIs([N])`: the attacker's type after effects) with
//! `then_discard`; the Tool is discarded only when the reduction applied.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "HabanBerry",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 60, subject: SlotPred::Holder, source: SlotPred::TypeIs(ct::DRAGON), then_discard: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
