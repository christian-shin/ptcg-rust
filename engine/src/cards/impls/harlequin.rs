//! Harlequin (WHT): each player shuffles their hand into their deck; flip a
//! coin: heads you draw 5 and your opponent draws 3, tails 3 and 5.
//!
//! Twinleaf: the Supporter moves to the supporter pile (preventDefault);
//! after the flip the player's hand (minus this card) and the opponent's
//! whole hand are moved with MoveCardsEffects (the opponent's can be
//! prevented, which skips their shuffle and draw), then SHUFFLE_DECK and
//! DRAW_CARDS for the opponent and the player.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Harlequin",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Coin(CoinSpec { heads: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })), Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })), Step::new(Op::ShuffleQueued(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck) })), Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) })), Step::new(Op::ShuffleQueued(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(5)) }))], tails: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })), Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })), Step::new(Op::ShuffleQueued(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck) })), Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(5)) })), Step::new(Op::ShuffleQueued(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))], ..CoinSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
