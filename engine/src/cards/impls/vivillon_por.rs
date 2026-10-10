//! Vivillon (POR / M3): Grand Wing — once during your turn, your opponent
//! shuffles their hand and puts it on the bottom of their deck; if they did,
//! they draw 4 cards. Blow Through — 60+; 60 more if a Stadium is in play.
//!
//! Twinleaf: throws POWER_ALREADY_USED (BIG_WINGS_MARKER) or, with an empty
//! opposing hand, CANNOT_USE_POWER. The hand is permuted in place by an
//! inline `Chance.shuffle(hand.length)`, moved into a fresh CardList, then to
//! the bottom of the deck, then MOVE_CARDS (count 4) deck → hand; the marker
//! and ABILITY_USED follow. The marker is removed at the owner's end of turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Vivillon@POR",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BIG_WINGS_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any)],
        // Your opponent shuffles their hand and puts it on the bottom of their deck; if they did, they draw 4 cards.
        steps: &[
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(4)) })),
        ],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(60, Cond::StadiumInPlay(Pred::Any)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
