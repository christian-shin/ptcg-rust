//! Jirachi ex (30C): Wish Granter — draw cards until you have 7 cards in your
//! hand. Swift — 150; not affected by Weakness or Resistance, or by effects
//! on the opponent's Active Pokémon.
//!
//! Twinleaf: DRAW_CARDS_UNTIL_CARDS_IN_HAND is a plain `deck.moveTo(hand, n)`
//! (no MoveCardsEffect). Swift uses THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS
//! with `ignoreWeaknessAndResistance` (phase 4b; Weakness and Resistance used
//! to apply, despite the text) and sets `ignoreDefenderEffects` (phase 4b R7B:
//! the damage used to be added straight to the Active, skipping the effects on
//! the attacker too).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jirachiex",
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
