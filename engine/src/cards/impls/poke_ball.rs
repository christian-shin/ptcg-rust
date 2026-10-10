//! Poké Ball (JU): flip a coin. If heads, you may search your deck for any
//! Basic Pokémon or Evolution card, show it to your opponent, and put it
//! into your hand. Shuffle your deck afterward.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokeBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Coin(CoinSpec {
                heads: &[Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    msg: "",
                    cancel: false,
                }))],
                ..CoinSpec::DEFAULT
            })),
            // Today's behavior kept: the deck is shuffled on tails too.
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
