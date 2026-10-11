//! Special Red Card (CRI 82): "You can use this card only if your opponent has 3 or fewer Prize cards remaining." Your opponent
//! shuffles their hand and puts it on the bottom of their deck. If they put any cards on the bottom of their deck in this way, they
//! draw 3 cards.
//!
//! The Prize count is the printed condition (`Cond::Printed`). The hand goes to the bottom shuffled (APR E-35) in one PutIntoDeck;
//! "if they put any" reads its outcome (`Cond::Done`, APR E-20). With an empty hand nothing is put, so nothing is drawn (id2365: it
//! can't be played).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpecialRedCard",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(3)))],
        steps: &[
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
            // "If they put any cards on the bottom of their deck in this way, they draw 3 cards" (id2365: the draw depends
            // on the put).
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
