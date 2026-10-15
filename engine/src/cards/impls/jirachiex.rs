//! Jirachi ex (30C): Wish Granter — draw cards until you have 7 cards in your hand. Swift — 150; this attack's damage isn't
//! affected by Weakness or Resistance, or by any effects on your opponent's Active Pokémon.
//!
//! Wish Granter is one Draw event of the attack's effect (nothing with 7 or more cards in the hand). Swift sets the attack's
//! three flags before the damage.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JirachiEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(7)) }))] },
        // Swift: not affected by Weakness, Resistance or effects on the Defending Pokémon.
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
