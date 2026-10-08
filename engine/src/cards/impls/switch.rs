//! Switch (BS): switch your Active Pokémon with 1 of your Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Switch",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
