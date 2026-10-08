//! Hop's Snorlax (JTG): Extra Helpings — attacks used by your Hop's Pokémon
//! do 30 more damage to your opponent's Active Pokémon (before Weakness and
//! Resistance); doesn't stack. Dynamic Press — 140; 80 damage to itself.
//!
//! Twinleaf: any DealDamageEffect whose attacker (`effect.player`) has this
//! card as the top Pokémon of a slot; after the ability-lock probe, adds 30
//! when the attacker's current Active is a Hop's Pokémon, the target is the
//! opponent's Active and the effect's `damageIncreased` flag is unset (then
//! sets it). The self-damage DealDamageEffect also passes through here.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsSnorlax",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(self_damage(80)),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, nonstacking: true, guard: Cond::Slot(SlotExpr::Active(Who::Me), SlotPred::Tag(tag::HOPS)), ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
