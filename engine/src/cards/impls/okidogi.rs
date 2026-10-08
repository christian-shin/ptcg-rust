//! Okidogi (TWM): Adrena-Power — if this Pokémon has any [D] Energy
//! attached, it gets +100 HP, and its attacks do 100 more damage to the
//! opponent's Active Pokémon (before Weakness and Resistance). Good Punch — 70.
//!
//! Twinleaf: on a DealDamageEffect whose source slot holds this card
//! (non-zero damage, target = the attacker's opponent's Active) and on a
//! CheckHpEffect whose target slot holds it; the lock check uses the
//! effect's player; [D] counts when an energy-map entry provides DARK or ANY.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Okidogi",
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::HpMod(HpModSpec { amount: 100, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Provides(ct::DARK)]), guard: Cond::True }),
        },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::DamageDealt(DamageDealtSpec {
                amount: 100,
                attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Provides(ct::DARK)]),
                side: Side::Any,
                needs_damage: true,
                ..DamageDealtSpec::DEFAULT
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
