//! Hop's Snorlax (JTG): Extra Helpings — attacks used by your Hop's Pokémon
//! do 30 more damage to your opponent's Active Pokémon (before Weakness and
//! Resistance); doesn't stack. Dynamic Press — 140; 80 damage to itself.
//!
//! Extra Helpings is a `DamageDealt` modifier read in the damage calculation while the Ability works: +30 to the
//! Damage event's amount when the attacking player's Active Pokémon is a Hop's Pokémon and the target is the opponent's
//! Active Pokémon, once however many Snorlax are in play. The recoil of Dynamic Press is one Damage event on the
//! attacker, never to the opponent's Active, so it gets none.
//!
//! An attack that does nothing or no damage gets no bonus (APR "does nothing", id675; scenarios hops-snorlax-jtg-117-*). JP FAQ Hop's
//! Snorlax: it adds up with Hop's Choice Band, and works against an opposing Active Pokémon with Hide 'n' Sneak (a modifier on the
//! attacker's side, not an effect on the target).
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
