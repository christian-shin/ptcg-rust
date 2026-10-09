//! Clefairy (M3 / POR 30): Follow Me — switch in 1 of your opponent's
//! Benched Pokémon. Flop — 30.
//!
//! With no Benched opponent Pokémon the attack is still usable and does nothing (ruling 1790). Rule: the switch-in is
//! a ChangeActive (SwitchIn) done to the Benched Pokémon chosen (APR C-05, id2155): a Pokémon protected from the
//! effects of attacks (Mist Energy, Unaware, Hide 'n' Sneak, ...) can be chosen, but it isn't switched in.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Clefairy",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
