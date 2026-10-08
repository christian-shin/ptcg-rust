//! Payapa Berry (SCR 141, Pokémon Tool): if the Pokémon this card is attached
//! to is damaged by an attack from your opponent's [P] Pokémon, it takes 60
//! less damage (after applying Weakness and Resistance), and discard this
//! card.
//!
//! Twinleaf (shared with Babiri Berry and Haban Berry): on a PutDamageEffect
//! whose target holds this tool, during the attack phase, unless the tool is
//! blocked for the owner or the source belongs to the same player, and (phase 4b
//! fix: it used to discard even when no damage was taken) unless the damage is
//! already 0, the effect is prevented or the target's preventDamage effect stops
//! this source: a CheckPokemonType on the source; if it contains the type, the
//! damage is reduced by 60 (floored at 0) and the tool moves from the target
//! slot to the owner's discard (no sourceCard).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "PayapaBerrySCRPool",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 60, subject: SlotPred::Holder, source: SlotPred::TypeIs(ct::PSYCHIC), then_discard: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
