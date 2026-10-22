//! Veluza (SCR 45): Food Prep - attacks used by this Pokémon cost [C] less for each Kofu card in your discard pile. Sonic
//! Edge - 110; this attack's damage isn't affected by any effects on your opponent's Active Pokémon.
//!
//! Food Prep is an Ability `AttackCost` reduction over this Pokémon's attacks, one [C] for each Kofu card in the discard
//! pile (floored at zero, APR D-11, D-12; Crabominable SCR 42). Sonic Edge sets the attack flag `IgnoreDefenderEffects`
//! before the damage (Druddigon WHT 71).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Veluza",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
            ],
        },
    ],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Attacks cost [C] less for each Kofu card in your discard pile.
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Trainer, Pred::Name("Kofu")]))),
            subject: SlotPred::IsThisPokemon,
            side: Side::Owner,
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
