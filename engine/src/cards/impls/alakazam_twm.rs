//! Alakazam (TWM): Strange Hacking — your opponent's Active Pokémon is now
//! Confused; you may move any number of damage counters from your opponent's
//! Pokémon to their other Pokémon in any way you like. Psychic — 10+, 50 more
//! for each Energy attached to your opponent's Active Pokémon.
//!
//! Strange Hacking's Confusion is a GainCondition(Confused) with the attack's
//! cause (an attack effect). The attack builds maxAllowedDamage from a
//! CheckHpEffect per opponent Pokémon and opens a MoveDamagePrompt (opponent's
//! Active + Bench, cancellable, defaults otherwise); each transfer moves 10
//! damage directly if the source has at least 10. Psychic counts
//! `provides` of the opponent's CheckProvidedEnergyEffect (their Active).
//!
//! Fixed (phase 4b, R7F-6; ruling 1665): the transfers bypassed Mist Energy
//! and Repelling Veil. Each one now probes the source and the destination with
//! a PutCountersEffect of 0 counters: a protected source keeps its counter, a protected
//! destination loses the counter that leaves the source.
//!
//! The prompt answers one transfer per damage counter, 20-30 of them for the
//! bot (any number is valid): `Res::DamageTransfers` is run-length encoded and
//! `damage_transfers` expands it (Y2-3; it held 16 transfers before).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Alakazam@TWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused])),
                Step::after_damage(Op::MoveCounters(MoveCountersSpec { kind: MoveCountersKind::AnyAmong { who: Who::Opp } })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(50)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
