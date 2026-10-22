//! Hoopa (ASC 141): Filch - draw 2 cards. Knuckle Impact - 130; during your next turn, this Pokémon can't use attacks.
//!
//! Filch is one Draw event of 2 cards (Binacle POR 42). Knuckle Impact arms a `CANT_ATTACK` lasting lock on this Pokémon until your next turn (N's Zekrom ASC 155).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Hoopa",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
