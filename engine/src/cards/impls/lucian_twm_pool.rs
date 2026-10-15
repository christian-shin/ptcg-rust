//! Lucian (TWM 157): Supporter. Each player shuffles their hand and puts it on the bottom of their deck. If either player
//! put any cards on the bottom of their deck in this way, each player flips a coin. If heads, that player draws 6 cards. If
//! tails, they draw 3 cards.
//!
//! Each hand goes to the bottom of its owner's deck in a random order by one PutIntoDeck event per player (cause: this
//! Supporter); then you flip and draw, then the opponent flips and draws (one Draw event each). The condition "if
//! either player put any cards" always holds: the card can't be played unless a hand holds a card (a Trainer that can't
//! have any effect can't be played, id925), and no lock refuses a hand card going into the deck, so no `Cond::Done`.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lucian",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Any(&[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any)])],
        steps: &[
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::Coin(CoinSpec { heads: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(6)) }))], tails: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))], ..CoinSpec::DEFAULT })),
            Step::new(Op::Coin(CoinSpec { flipper: Who::Opp, heads: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(6)) }))], tails: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) }))], ..CoinSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
