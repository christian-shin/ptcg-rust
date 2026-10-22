//! Team Rocket's Great Ball (DRI 175, Item): flip a coin. If heads, search your deck for an Evolution Team Rocket's Pokémon, reveal
//! it, and put it into your hand. If tails, search your deck for a Basic Team Rocket's Pokémon, reveal it, and put it into your hand.
//! Then, shuffle your deck.
//!
//! The flip always happens (a printed coin flip is never gated on its consequence); the search is for 0 or 1 card (the search can
//! be declined) of a Pokémon with the Team Rocket's tag that is an Evolution Pokémon (heads) or a Basic Pokémon (tails), revealed,
//! then the deck is shuffled once (APR E-19).
use crate::spec::prelude::*;
use crate::types::tag;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsGreatBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Coin(CoinSpec {
                heads: &[Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: DECK, predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(tag::TEAM_ROCKET), Pred::Not(&Pred::Basic)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: true,
                }))],
                tails: &[Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: DECK, predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(tag::TEAM_ROCKET), Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: true,
                }))],
                ..CoinSpec::DEFAULT
            })),
            Step::new(Op::If(IfSpec { cond: Cond::Nonempty(DECK, Pred::Any), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
