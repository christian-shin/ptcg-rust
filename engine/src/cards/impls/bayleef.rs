//! Bayleef (M1S): Push Down — 50. Switch out your opponent's Active Pokémon
//! to the Bench (your opponent chooses the new Active Pokémon).
//!
//! AFTER_ATTACK → SWITCH_OUT_OPPONENT_ACTIVE_POKEMON(player, { sourceEffect }):
//! a SwitchOutOpponentsActiveEffect probe (preventable, e.g. Mist Energy)
//! before the opponent's ChoosePokemonPrompt, then another one carrying the
//! chosen bench target, whose reducer does the switch. Each effect is built
//! on a fresh AttackEffect (source = the player's Active at that time).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bayleef",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Opp, kind: SwitchKind::SwitchOut, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
