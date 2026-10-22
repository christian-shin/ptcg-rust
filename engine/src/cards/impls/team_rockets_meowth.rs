//! Team Rocket's Meowth (DRI 149): Paw-cket Pilfer - choose a random card from your opponent's hand; your opponent reveals
//! that card and shuffles it into their deck. Fury Swipes - 20x; flip 3 coins, 20 damage for each heads.
//!
//! Paw-cket Pilfer: the random card goes from the opponent's hand into their deck, revealed to you, and their deck is
//! shuffled (APR E-19; Snorunt TWM 51). Fury Swipes flips before the damage, 20 per heads (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMeowth",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any),
                    yes: &[
                        Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), position: DeckPosition::Bottom, reveal: Some(Who::Me), ..PutIntoDeckSpec::DEFAULT })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(3), per_heads: PerHeads::DamageIs(20), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
