//! Sylveon (PRE): Safeguard - prevent all damage done to this Pokémon by
//! attacks from your opponent's Pokémon ex. Magical Shot - 100.
//!
//! Twinleaf: any PutDamageEffect on a slot holding this card (with this card
//! on top) during the ATTACK phase, from another player's ex Pokémon (source
//! top card tagged `POKEMON_ex`), is prevented unless the Ability is blocked.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Sylveon",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::PreventDamage(PreventDamageSpec {
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            source: SlotPred::Tag(tag::POKEMON_EX_LOWER),
            ..PreventDamageSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
