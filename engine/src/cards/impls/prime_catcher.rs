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
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true })),
            // If you do, switch your Active Pokémon with 1 of your Benched Pokémon.
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
