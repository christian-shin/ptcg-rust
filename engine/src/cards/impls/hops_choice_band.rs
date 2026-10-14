//! Hop's Choice Band (JTG 148, Tool): attacks used by the Hop's Pokémon this card is attached to cost [C] less and do 30 more
//! damage to your opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! Two Tool passives, each bound to the Pokémon holding this card and a Hop's Pokémon (`Holder` and the Hop's tag):
//! `AttackCost` ([C] less, summed with the other changes and floored at zero, D-11 / D-12) and `DamageDealt` (+30 in the
//! attacker-side pass of the damage calculation, before Weakness: only when the attack does damage and the target is the
//! opponent's Active Pokémon, id675, APR "does nothing"). Both work while the Tool does (`tools_off`: Jamming Tower). They add up
//! with Hop's Snorlax's Extra Helpings (JP FAQ Hop's Choice Band: +30 and +30).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsChoiceBand",
    passives: &[
        Passive { origin: RuleSource::Tool, modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::Reduce(Num::Lit(1)), subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::HOPS)]), ..AttackCostSpec::DEFAULT }) },
        Passive { origin: RuleSource::Tool, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::HOPS)]), needs_damage: true, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
