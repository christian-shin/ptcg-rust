//! Team Rocket's Diglett (ASC 100): Relentless Burrowing - flip a coin until you get tails. For each heads, discard the top card of
//! your opponent's deck.
//!
//! The coins are flipped until tails (this attack does no damage), then the top cards of the opponent's deck, as many as there
//! were heads, are one Discard event caused by this attack (Tyranitar JTG's Cracking Stomp).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsDiglett",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::UntilTails,
            then: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Heads), ..DiscardSpec::DEFAULT }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
