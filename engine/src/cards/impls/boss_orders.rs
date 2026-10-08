//! Boss's Orders (PAL): switch 1 of the opponent's Benched Pokémon with
//! their Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BossOrders",
    // Switch 1 of your opponent's Benched Pokémon with their Active Pokémon (it can't be played
    // when it would not change the game state; used through an attack it does nothing then).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
