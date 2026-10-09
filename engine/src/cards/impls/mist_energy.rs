//! Mist Energy (TEF): provides [C]. Prevent all effects of attacks from your
//! opponent's Pokémon done to the Pokémon this card is attached to (damage is
//! not an effect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistEnergy",
    passives: &[
        // An opponent's attack switching the Pokémon this card is attached to in or out (ChangeActive: APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Energy, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::Holder, CHANGE_ACTIVE_BY_OPP_ATTACK)) },Passive {
        origin: RuleSource::Energy,
        modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec { subject: SlotPred::Holder, abilities: false, probe_for_attacker: true, needs_source_pokemon: true, ..PreventAttackEffectsSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
