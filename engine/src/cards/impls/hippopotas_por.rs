//! Hippopotas (POR 39): Sand Attack - 10; during your opponent's next turn, if the Defending Pokémon tries to use an attack, your
//! opponent flips a coin. If tails, that attack doesn't happen. Bite - 60.
//!
//! Sand Attack leaves a coin-gated lock on the Defending Pokémon (`passive::ATTACK_COIN_GATE`, `CoinGate::TailsStopsAttack`): when it
//! tries to use an attack its player flips first, before the Confusion flip (id2317) and before the attack's own costs (id497); tails,
//! the attack ends at once and every effect of it is stopped, heads it goes on (id606). The effect is on the Defending Pokémon, not on
//! the player: retreating clears it (APR C-15) and a Pokémon that moves up isn't affected (id1774).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HippopotasPOR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&ATTACK_COIN_GATE)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
