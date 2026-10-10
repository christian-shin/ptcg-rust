//! Leafeon ex (PRE 6, Tera): Verdant Storm — 60× damage for each Energy
//! attached to all of your opponent's Pokémon. Moss Agate — 230; heal 100 damage
//! from each of your Benched Pokémon. Tera: as long as this Pokémon is on your
//! Bench, prevent all damage done to it by attacks.
//!
//! Rule: Verdant Storm counts the Energy cards attached (not the types they
//! provide); Moss Agate is a Heal (RemoveCounters event, cause: this attack) on each
//! of your Benched Pokémon after the damage. The Tera rule is `TERA_RULE`.
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
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
