//! Mist Energy (TEF): provides [C]. Prevent all effects of attacks from your
//! opponent's Pokémon done to the Pokémon this card is attached to (damage is
//! not an effect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistEnergy",
    passives: &[
        // Prevent all effects of attacks used by your opponent's Pokémon done to the Pokémon this card is attached to (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155; damage is not an effect).
        Passive { origin: RuleSource::Energy, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::Holder, EFFECTS_OF_OPP_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
