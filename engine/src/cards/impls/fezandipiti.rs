//! Fezandipiti (TWM): Adrena-Pheromone — if this Pokémon has any [D] Energy
//! attached and is damaged by an attack, flip a coin; if heads, prevent that
//! damage. Energy Feather — 30 damage for each Energy attached to this
//! Pokémon.
//!
//! Twinleaf: Adrena-Pheromone runs on every PutDamageEffect whose target slot
//! holds this card (not necessarily on top): it needs this card on top and the
//! attack phase, then IS_ABILITY_BLOCKED and a CheckProvidedEnergyEffect are
//! both evaluated with the OWNER as `player` (phase 4b: it used to be the
//! attacker). [D] or a rainbow unit counts. The damage must be positive; a
//! CoinFlipEffect (owner, no callback) decides.
//! Energy Feather counts every provided unit on the slot holding this card.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fezandipiti",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::ProvidedUnits), &Num::Lit(30)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::CoinFlip, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::Provides(ct::DARK)]), ..PreventDamageSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
