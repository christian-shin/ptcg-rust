//! Leafeon ex (PRE 6): Verdant Storm — 60 damage for each Energy attached to
//! all of your opponent's Pokémon. Moss Agate — 230; heal 100 damage from
//! each of your Benched Pokémon. Tera: no attack damage on the Bench.
//!
//! Twinleaf: Verdant Storm counts CheckProvidedEnergyEffect `energyMap`
//! entries (cards, not provided types) and sets `effect.damage`; Moss
//! Agate reduces a HealTargetEffect per Benched Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Leafeonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedCards), &Num::Lit(60)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Each(SlotSel::Bench(Who::Me)), hp: Num::Lit(100), via: HealVia::Attack, clear_conditions: false })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
