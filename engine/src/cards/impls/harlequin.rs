//! Harlequin (WHT): each player shuffles their hand into their deck. Then,
//! flip a coin: heads you draw 5 and your opponent draws 3, tails 3 and 5.
//!
//! The hands are shuffled in first (the player's hand without this card, the
//! opponent's whole hand), then the coin is flipped, then each player draws.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Harlequin",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(0) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(0) })),
            Step::new(Op::Coin(CoinSpec {
                heads: &[
                    Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(5)) })),
                    Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) })),
                ],
                tails: &[
                    Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
                    Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(5)) })),
                ],
                ..CoinSpec::DEFAULT
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
