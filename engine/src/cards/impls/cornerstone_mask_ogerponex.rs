//! Cornerstone Mask Ogerpon ex (TWM 112): Cornerstone Stance — prevent all
//! damage done to this Pokémon by attacks from your opponent's Pokémon that
//! have an Ability. Demolish — 140, not affected by Weakness, Resistance or
//! effects on the opponent's Active. Tera.
//!
//! Twinleaf: Demolish sets `ignoreDefenderEffects` and ignores Weakness and
//! Resistance on the AttackEffect (phase 4b R7B: it used to add 140 straight
//! to the Active, skipping the attacker's effects). Cornerstone
//! Stance checks the source's printed `powers` (any kind) and an ability
//! probe for this card's owner. The Tera bench protection is a card rule
//! (printed above the Ability), so Ability locks don't touch it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CornerstoneMaskOgerponex",
    // Demolish: not affected by Weakness, Resistance or effects on the Defending Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
        ],
    }],
    passives: &[
        // Cornerstone Stance: attacks from Pokémon that have an Ability.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Cause(CausePred::Pokemon(SlotPred::PrintsPower))]))),
        },
        // Tera: a card rule, not part of the Ability (it stays while the Ability is locked).
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
