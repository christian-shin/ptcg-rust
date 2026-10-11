//! Poké Ball (JU 64): flip a coin. If heads, you may search your deck for any Basic Pokémon or Evolution card. Show that
//! card to your opponent, then put it into your hand. Shuffle your deck afterward.
//!
//! Usable with a card in the deck (id840). On heads the chosen Pokémon is shown and put into the hand (PutIntoHand, this
//! Item its cause); the deck is shuffled afterward on heads and on tails, as printed ("Shuffle your deck afterward" isn't
//! part of the heads branch).
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
            // "Shuffle your deck afterward": after the coin, on heads (with or without a card found) and on tails.
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
