//! Pokémon Catcher (SVI, as Pokemon Catcher POR): flip a coin; if heads,
//! switch 1 of your opponent's Benched Pokémon with their Active Pokémon.
//!
//! With no Benched Pokémon of the opponent the switch can't change the game and the flip alone isn't what the card does
//! (id925): the rules' walk (`spec::run::usable_by_rules`) doesn't offer it; no `needs` entry restates that.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokemonCatcherPOR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, required: false }))],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
