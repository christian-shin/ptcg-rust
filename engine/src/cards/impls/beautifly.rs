//! Beautifly (ASC 13): Stun Spore - 40; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Energy Straw - 80x;
//! your opponent reveals their hand, and this attack does 80 damage for each Energy card you find there.
//!
//! Stun Spore: the coin after the damage, heads inflicts Paralyzed (Mareep CRI 27). Energy Straw: the hand is always revealed (even
//! an empty one: the text says "reveals their hand"), then the damage is 80 times the Energy cards in it, before the damage
//! (Whimsicott ex WHT 5's Wondrous Cotton).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Beautifly",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })),
                Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Energy), &Num::Lit(80)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
