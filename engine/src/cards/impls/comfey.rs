//! Comfey (SCR): Flower Shower — each player draws 3 cards: a Draw event for each player, by the attack (as many as the
//! deck has, APR H; an attack is usable with an empty deck, APR D-01). Play Rough — 20+; flip a coin, if heads 20 more
//! damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Comfey",
    attacks: &[
        // Flower Shower: each player draws 3 cards (fewer when the deck has fewer).
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(3)) })),
            ],
        },
        // Play Rough: flip a coin, if heads 20 more damage.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
