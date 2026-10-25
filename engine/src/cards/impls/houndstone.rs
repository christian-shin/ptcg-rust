//! Houndstone (MEG 66): Horrifying Bite - 30; flip a coin until you get tails. For each heads, choose a random card from your
//! opponent's hand. Your opponent reveals those cards and shuffles them into their deck. Hammer In - 130.
//!
//! After the damage the flips are made; then as many random cards of the opponent's hand as heads (all of it when fewer) are revealed
//! to the player and put into the opponent's deck (a PutIntoDeck caused by the attack, Banette JTG 60's put), and that deck is
//! shuffled (APR E-19). With no heads or an empty hand the cards are none, the deck is still shuffled.
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "Houndstone",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::UntilTails,
            then: &[
                Step::new(Op::PutIntoDeck(PutIntoDeckSpec {
                    from: OPP_HAND,
                    cards: CardSel::Random(Num::Min(&Num::Heads, &Num::ZoneSize(OPP_HAND))),
                    position: DeckPosition::Bottom,
                    reveal: Some(Who::Me),
                    ..PutIntoDeckSpec::DEFAULT
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
            ],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
