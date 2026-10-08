//! Thick Scale (ASC 211, Pokémon Tool): the [N] Pokémon this card is attached
//! to takes 50 less damage from attacks from your opponent's [G], [R], [W],
//! or [L] Pokémon (after applying Weakness and Resistance).
//!
//! Twinleaf: on a PutDamageEffect whose target holds this tool, during the
//! attack phase, unless the tool is blocked for the owner or the source
//! belongs to the same player: a CheckPokemonType on the holder must contain
//! [N], then one on the source must contain [G], [R], [W] or [L]; the damage is
//! reduced by 50 (floored at 0). The card stays attached.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "ThickScaleASCPool",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 50, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::DRAGON)]), source: SlotPred::OneOf(&[SlotPred::TypeIs(ct::GRASS), SlotPred::TypeIs(ct::FIRE), SlotPred::TypeIs(ct::WATER), SlotPred::TypeIs(ct::LIGHTNING)]), ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
