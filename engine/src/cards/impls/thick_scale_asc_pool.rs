//! Thick Scale (ASC 211, Pokémon Tool): the [N] Pokémon this card is attached to takes 50 less damage from attacks from your
//! opponent's [G], [R], [W], or [L] Pokémon (after applying Weakness and Resistance).
//!
//! A `DamageTaken` modifier of 50, subject the Holder while it is [N], source an attacking Pokémon of one of the four types (read
//! after effects; one reduction even for a two-type attacker). It applies after Weakness and Resistance and the Tool stays attached.
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
