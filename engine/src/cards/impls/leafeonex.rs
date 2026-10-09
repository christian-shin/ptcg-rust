//! Leafeon ex (PRE 6): Verdant Storm — 60 damage for each Energy attached to
//! all of your opponent's Pokémon. Moss Agate — 230; heal 100 damage from
//! each of your Benched Pokémon. Tera: no attack damage on the Bench.
//!
//! Twinleaf: Verdant Storm counts CheckProvidedEnergyEffect `energyMap`
//! entries (cards, not provided types) and sets `effect.damage`; Moss
//! Agate is a RemoveCounters (heal) of 100 per Benched Pokémon, caused by the attack.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Leafeonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedCards), &Num::Lit(60)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Bench(Who::Me), body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(100), clear_conditions: false }))] })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
