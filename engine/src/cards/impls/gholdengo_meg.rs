//! Gholdengo (MEG 99): All-You-Can-Grab - flip a coin until you get tails. Search your deck for a number of cards up to the number of
//! heads and put them into your hand. Then, shuffle your deck. Speed Attack - 100. This is a different card from Gholdengo SSP 131.
//!
//! The flips come first (until tails), then one search of between 0 and the heads (at most the deck's size) cards of any kind into the
//! hand, then the shuffle, which is done whatever was taken (APR E-19); with no heads the search is skipped and the deck is still
//! shuffled (Mega Audino ex ASC 109's form).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "GholdengoMEG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::UntilTails,
            then: &[
                Step::new(Op::If(IfSpec {
                    cond: Cond::All(&[Cond::Cmp(Num::Heads, CmpOp::Gt, Num::Lit(0)), Cond::Nonempty(DECK, Pred::Any)]),
                    yes: &[Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: DECK, predicate: Pred::Any, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Heads, &Num::ZoneSize(DECK)) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: false },
                        cancel: false,
                    }))],
                    no: &[],
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
