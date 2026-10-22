//! Golbat (CRI 50): Covert Flight - 30; during your opponent's next turn, prevent all damage done to this Pokémon by attacks from Basic Pokémon.
//!
//! A lasting `Prevent` over damage from Basic Pokémon (Dipplin SCR 13).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "Golbat",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(Stage::Basic)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
