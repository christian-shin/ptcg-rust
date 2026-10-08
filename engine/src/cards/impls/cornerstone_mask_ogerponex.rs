//! Cornerstone Mask Ogerpon ex (TWM 112): Cornerstone Stance — prevent all
//! damage done to this Pokémon by attacks from your opponent's Pokémon that
//! have an Ability. Demolish — 140, not affected by Weakness, Resistance or
//! effects on the opponent's Active. Tera.
//!
//! Twinleaf: Demolish sets `ignoreDefenderEffects` and ignores Weakness and
//! Resistance on the AttackEffect (phase 4b R7B: it used to add 140 straight
//! to the Active, skipping the attacker's effects). Cornerstone
//! Stance checks the source's printed `powers` (any kind) and an ability
//! probe for this card's owner. The Tera bench protection sits after it and
//! is skipped by its early returns (this card not the top card, no source
//! Pokémon, own damage, outside the attack phase, or the ability blocked).
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
            modifier: Modifier::PreventDamage(PreventDamageSpec {
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
                source: SlotPred::PrintsPower,
                ..PreventDamageSpec::DEFAULT
            }),
        },
        // Tera: no attack damage while Benched. Today's behavior kept (I-PC1): skipped when the
        // Ability's own gates are.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
