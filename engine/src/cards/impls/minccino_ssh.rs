//! Minccino (SSH): Glance — look at the top card of your opponent's deck.
//! Tail Slap — flip 2 coins, 20 damage for each heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Minccino@SSH",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: 0, ..LookSpec::DEFAULT })),
                Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Me, Zone::Scratch(0))), by: Who::Me, to: Who::Me, when_empty: false })),
                Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), position: DeckPosition::Top, ..PutIntoDeckSpec::DEFAULT })),
            ],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(20), ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
