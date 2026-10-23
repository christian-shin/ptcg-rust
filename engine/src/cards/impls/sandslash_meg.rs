//! Sandslash (MEG 69): Sand Attack - 50; during your opponent's next turn, if the Defending Pokémon tries to use an attack,
//! your opponent flips a coin. If tails, that attack doesn't happen. Mud Shot - 100.
//!
//! Sand Attack leaves the coin-gated lock on the Defending Pokémon (`passive::ATTACK_COIN_GATE`, Hippopotas POR 39): its
//! player flips when it tries to attack, before the Confusion flip (id2317) and before the attack's costs (id497); tails
//! ends the attack and stops all its effects, heads it goes on (id606); the effect is on the Defending Pokémon, so
//! retreating clears it and a Pokémon that moves up isn't affected (id1774).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SandslashMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&ATTACK_COIN_GATE)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
