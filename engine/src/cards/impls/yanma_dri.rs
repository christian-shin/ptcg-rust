//! Yanma (DRI): Whirlwind — switch out your opponent's Active Pokémon to the
//! Bench (your opponent chooses the new Active Pokémon). Razor Wing — 30.
//!
//! AFTER_ATTACK → SWITCH_OUT_OPPONENT_ACTIVE_POKEMON(player, { sourceEffect })
//! (same flow as Bayleef M1S): a preventable SwitchOutOpponentsActiveEffect
//! probe, the opponent's ChoosePokemonPrompt, then another effect carrying
//! the chosen bench target. Each effect is built on a fresh AttackEffect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yanma@DRI",
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
