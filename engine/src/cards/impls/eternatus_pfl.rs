//! Eternatus (PFL 69): Shatter — 50; discard a Stadium in play. Power Rush — 130; flip a coin. If tails, during your next turn,
//! this Pokémon can't use attacks.
//!
//! Shatter discards the Stadium in play after the damage (a LeavePlay to its owner's discard pile, caused by the attack; nothing
//! happens with no Stadium). Power Rush flips after the damage; on tails a lasting lock on this Pokémon's attacks lasts through
//! your next turn (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EternatusPFL",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(DISCARD_STADIUM)] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Coin(CoinSpec {
                tails: &[Step::new(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))],
                ..CoinSpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
