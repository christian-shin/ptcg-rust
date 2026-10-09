//! Fezandipiti (TWM): Adrena-Pheromone — if this Pokémon has any [D] Energy
//! attached and is damaged by an attack, flip a coin; if heads, prevent that
//! damage. Energy Feather — 30 damage for each Energy attached to this
//! Pokémon.
//!
//! Rule: Adrena-Pheromone applies to damage from an attack to the slot holding
//! this card: it needs this card on top and the attack phase, the Ability not
//! blocked and a [D] (or rainbow) unit provided, both read with the OWNER as
//! `player`. The damage must be positive; a CoinFlip event (owner) decides.
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
