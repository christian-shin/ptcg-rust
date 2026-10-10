//! Special Red Card (M4): usable only if the opponent has 3 or fewer Prize
//! cards left; they put their hand on the bottom of their deck and, if any
//! cards were put there, draw 3 cards.
//!
//! Phase 4b (ruling n=1833): can't be played when the opponent has no cards in
//! hand. Audit aud-d (Advanced Rulebook E-35): the hand is shuffled (`Chance.shuffle`) before it goes to the bottom.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpecialRedCard",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(3))],
        steps: &[
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
