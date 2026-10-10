//! Cornerstone Mask Ogerpon ex (TWM 112 / PRE 58): Cornerstone Stance —
//! prevent all damage from attacks done to this Pokémon by your opponent's
//! Pokémon that have an Ability. Demolish — 140; this attack's damage isn't
//! affected by Weakness or Resistance, or by any effects on your opponent's
//! Active Pokémon. Tera: as long as this Pokémon is on your Bench, prevent all
//! damage done to it by attacks.
//!
//! Rule: Demolish sets the attack flags IgnoreDefenderEffects, NoWeakness and
//! NoResistance (Shred: step 6's preventions on the Defending Pokémon don't
//! apply to it, APR C-16). Cornerstone Stance is a `Prevent` over `Kind(Damage)`
//! with the attacker as the cause (`CausePred::Pokemon(PrintsPower)`, the causing
//! Pokémon where it is now); it is an Ability, so an Ability lock turns it off.
//! The Tera rule is `TERA_RULE`, a card rule that Ability locks don't touch.
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
