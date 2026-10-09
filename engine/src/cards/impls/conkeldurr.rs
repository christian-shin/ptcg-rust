//! Conkeldurr (TWM 105): Tantrum — 80; this Pokémon is now Confused. Gutsy
//! Swing — 250; if this Pokémon is affected by a Special Condition, ignore
//! all Energy in this attack's cost.
//!
//! Tantrum's Confusion is a GainCondition(Confused) on this Pokémon with the
//! attack's cause; Gutsy Swing empties the CheckAttackCostEffect cost when
//! this card is the player's Active and it has any Special Condition. R7F-11
//! (rulings 252, 1552): the cost is also marked as set, so an increase (Rillaboom's
//! Drum Beating, ...) no longer adds a [C] back.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Conkeldurr",
    // Tantrum: this Pokémon is now Confused.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Conditions(ConditionsSpec {
            target: MY_ACTIVE,
            change: ConditionChange::Add(&[SpecialCondition::Confused]),
            gate: Gate::None,
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
