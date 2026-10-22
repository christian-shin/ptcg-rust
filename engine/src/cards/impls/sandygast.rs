//! Sandygast (TWM 91): Sand Attack - 10; during your opponent's next turn, if the Defending Pokémon tries to use an attack, your
//! opponent flips a coin. If tails, that attack doesn't happen. Spooky Shot - 30.
//!
//! The coin-gated lock of Hippopotas POR's Sand Attack (`passive::ATTACK_COIN_GATE`): the flip comes before the Confusion flip
//! (id2317); tails, the attack doesn't happen.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sandygast",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&ATTACK_COIN_GATE)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
