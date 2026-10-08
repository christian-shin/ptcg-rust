//! Repel (SUM / MEG): your opponent switches their Active Pokémon with 1 of
//! their Benched Pokémon.
//!
//! Twinleaf: throws CANNOT_PLAY_THIS_CARD without an opposing Bench, then
//! SWITCH_OUT_OPPONENT_ACTIVE_POKEMON (no sourceEffect, no cancel): the
//! *opponent* answers a ChoosePokemonPrompt over their own Bench and
//! `opponent.switchPokemon(selected[0], store, state)` runs in the callback.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Repel",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Opp, kind: SwitchKind::Plain, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
