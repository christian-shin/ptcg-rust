//! Pokémon Catcher (SVI, as Pokemon Catcher POR): flip a coin; if heads,
//! switch 1 of your opponent's Benched Pokémon with their Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokemonCatcher@POR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::Silent, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
