//! Stunfisk (ASC 62): Pouncing Trap — 30; during your opponent's next turn the Defending Pokémon can't retreat. During your next
//! turn, the Defending Pokémon takes 100 more damage from attacks (after applying Weakness and Resistance).
//!
//! Two lasting effects on the Defending Pokémon: a retreat lock until the end of the opponent's next turn and
//! `Lasting::TakesMoreDamage(100)` for your next turn, applied after Weakness and Resistance to every attack of that turn. Mist
//! Energy attached afterwards doesn't remove them (the Pokémon was already affected); both end when it moves to the Bench (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Stunfisk",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesMoreDamage(100) })),
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
