//! Energy Coin (BLK 81, Item): flip 2 coins. If both of them are heads, search your deck for a Basic Energy card and attach
//! it to 1 of your Pokémon. Then, shuffle your deck.
//!
//! The 2 flips are one sequence (`Flips::Count(2)`); once they are done, 2 heads search for 0 or 1 Basic Energy attached to
//! one Pokémon of yours, then the shuffle (APR E-19, as the Ogerpon attacks). The flips always happen: the card is playable
//! with a card in the deck (the coin is the effect).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "EnergyCoin",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(2)),
            then: &[Step::new(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Cmp(Num::Heads, CmpOp::Eq, Num::Lit(2)), Cond::Nonempty(DECK, Pred::Any)]),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: DECK, predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::AttachToPicked,
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
                ],
                no: &[],
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
