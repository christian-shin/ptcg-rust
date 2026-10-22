//! Okidogi (PRE 57): Adrena-Power - if this Pokémon has any [D] Energy attached, it gets +100 HP, and the attacks it uses do 100
//! more damage to your opponent's Active Pokémon (before applying Weakness and Resistance). Good Punch - 70.
//!
//! Two Ability declarations whose subject is this Pokémon (`Holder`) while it provides [D] (the Energy provided: Prism Energy,
//! Neo Upper Energy and the like count): an `HpMod` of +100 and a `DamageDealt` of +100 on the opponent's Active Pokémon, added
//! only to damage that is above 0 (APR B-01, C-16).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Okidogi",
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::HpMod(HpModSpec { amount: Num::Lit(100), subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Provides(ct::DARK)]), guard: Cond::True }),
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
