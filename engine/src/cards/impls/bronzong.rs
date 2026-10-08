//! Bronzong (TEF): Evolution Jammer — 30; during your opponent's next turn,
//! they can't play Pokémon from their hand to evolve. Super Psy Bolt — 100.
//!
//! Twinleaf: OPPONENT_CANNOT_EVOLVE_POKEMON is a PlayLockEffect with the
//! `evolve` flag (player-level).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bronzong",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(Locked::Evolve) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
