//! Comfey (SCR): Flower Shower — each player draws 3 cards (MOVE_CARDS
//! count 3, so fewer cards simply move fewer; phase 4b: it threw
//! CANNOT_USE_ATTACK when either deck was empty). Play Rough — 20+; flip a
//! coin, if heads 20 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Comfey",
    attacks: &[
        // Flower Shower: each player draws 3 cards (fewer when the deck has fewer).
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Move(MoveSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    to: ZoneRef(Who::Me, Zone::Hand),
                    cards: CardSel::Top(Num::Lit(3)),
                    ..MoveSpec::DEFAULT
                })),
                Step::after_damage(Op::Move(MoveSpec {
                    from: ZoneRef(Who::Opp, Zone::Deck),
                    to: ZoneRef(Who::Opp, Zone::Hand),
                    cards: CardSel::Top(Num::Lit(3)),
                    ..MoveSpec::DEFAULT
                })),
            ],
        },
        // Play Rough: flip a coin, if heads 20 more damage.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
