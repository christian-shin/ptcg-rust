//! Prime Catcher (TEF): switch in 1 of your opponent's Benched Pokémon to the
//! Active Spot. If you do, switch your Active Pokémon with 1 of your Benched
//! Pokémon.
//!
//! Twinleaf: with an empty opposing Bench the play fails (undefined state).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PrimeCatcher",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::Silent, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true })),
            Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Silent, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
