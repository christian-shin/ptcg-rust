//! Conkeldurr (TWM 105): Tantrum — 80; this Pokémon is now Confused. Gutsy Swing — 250; if this Pokémon is affected by a Special
//! Condition, ignore all Energy in this attack's cost.
//!
//! Tantrum's Confusion is a GainCondition(Confused) on this Pokémon with the attack as its cause. Gutsy Swing's cost is a
//! set cost (`SetCost(&[])`, "ignore all" is applied before "more" changes, APR D-13), for this Pokémon's own attack only
//! (Holder), while it has a Special Condition.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Conkeldurr",
    // Tantrum: this Pokémon is now Confused.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Conditions(ConditionsSpec {
            target: MY_ACTIVE,
            change: ConditionChange::Add(&[SpecialCondition::Confused]),
            when: Cond::True,
        }))],
    }],
    // Gutsy Swing: if this Pokémon is affected by a Special Condition, ignore all Energy in this
    // attack's cost.
    passives: &[Passive {
        origin: RuleSource::CardRule,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::SetCost(&[]),
            attack: Some(1),
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::HasCondition]),
            side: Side::Owner,
            guard: Cond::True,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
