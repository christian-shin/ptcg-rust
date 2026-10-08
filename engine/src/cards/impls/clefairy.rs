//! Clefairy (M3 / POR 30): Follow Me — switch in 1 of your opponent's
//! Benched Pokémon. Flop — 30.
//!
//! Twinleaf: AFTER_ATTACK; with no Benched opponent Pokémon the attack is still
//! usable and does nothing (phase 4b R7E, ruling 1790: it used to throw
//! CANNOT_USE_ATTACK); GUST_OPPONENT_BENCHED_POKEMON(player, { sourceEffect }) prompts the
//! attacker (no cancel) and then reduces a GustOpponentBenchEffect built on a
//! fresh AttackEffect (preventable, e.g. Mist Energy), whose reducer does the
//! switch.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Clefairy",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::Gust, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
