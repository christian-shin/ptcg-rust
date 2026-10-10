//! Stunfisk (M2a): Pouncing Trap — 30; during your opponent's next turn the
//! Defending Pokémon can't retreat. During your next turn, the Defending
//! Pokémon takes 100 more damage from attacks (after Weakness/Resistance).
//!
//! Twinleaf: DEFENDING_POKEMON_TAKES_MORE_DAMAGE_DURING_YOUR_NEXT_TURN(100)
//! (a DefendingPokemonTakesMoreDamageDuringAttackerNextTurnEffect on the
//! opponent's Active), then BLOCK_RETREAT (PreventRetreatEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Stunfisk@ASC",
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
