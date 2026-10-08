//! Lucian (TWM 157, Supporter): each player shuffles their hand and puts it
//! on the bottom of their deck. If either player put any cards on the bottom
//! of their deck in this way, each player flips a coin. If heads, that player
//! draws 6 cards. If tails, they draw 3 cards.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED, then CANNOT_PLAY_THIS_CARD when
//! both hands (excluding this card) are empty. Each hand is permuted with
//! `Chance.shuffle(n)` (player first, then opponent) and moved to the end of
//! the deck by MOVE_CARDS (empty hands skip both the shuffle and the move).
//! Then the player flips, then the opponent (in the first flip's callback);
//! both draw afterwards (player first). Note the coin flips happen even though
//! the text only requires them when cards were moved; the throw above makes
//! that always true.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LucianTWMPool",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Any(&[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any)])],
        steps: &[
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, shuffle_first: true, ..MoveSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::All, shuffle_first: true, ..MoveSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::Coin(CoinSpec { heads: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(6)) }))], tails: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))], ..CoinSpec::DEFAULT })),
            Step::new(Op::Coin(CoinSpec { flipper: Who::Opp, heads: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(6)) }))], tails: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) }))], ..CoinSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
