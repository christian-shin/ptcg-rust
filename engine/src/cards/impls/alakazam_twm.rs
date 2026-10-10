//! Alakazam (TWM): Strange Hacking — your opponent's Active Pokémon is now
//! Confused; you may move any number of damage counters from your opponent's
//! Pokémon to their other Pokémon in any way you like. Psychic — 10+, 50 more
//! for each Energy attached to your opponent's Active Pokémon.
//!
//! Strange Hacking's Confusion is a Special Condition event with the attack as its cause. The move is one MoveCounters
//! event (one action, both ends asked, APR C-08): Patrat's Watchful Eye stops it and the counters stay (id2350); per
//! pair a protected source keeps its counters and a protected destination makes them vanish (Mist Energy, Repelling Veil:
//! id390, id2150, id2192, id2257; Battle Cage on a Benched destination, JP FAQ Battle Cage). The prompt opens over
//! the opponent's Active and Bench (cancellable). Psychic counts the Energy the opponent's Active Pokémon provides.
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
