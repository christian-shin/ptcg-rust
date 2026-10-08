//! Meddling Memo (SSP): your opponent counts the cards in their hand,
//! shuffles them, and puts them on the bottom of their deck. If they do,
//! they draw that many cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MeddlingMemo",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Move(MoveSpec {
                from: ZoneRef(Who::Opp, Zone::Hand),
                to: ZoneRef(Who::Opp, Zone::Deck),
                cards: CardSel::All,
                into: Some(0),
                shuffle_first: true,
                ..MoveSpec::DEFAULT
            })),
            Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::RegCount(0)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
