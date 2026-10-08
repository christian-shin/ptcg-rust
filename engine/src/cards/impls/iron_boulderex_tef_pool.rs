//! Iron Boulder ex (TEF 99): Repulsor Axe — 60; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if it is Knocked Out),
//! put 8 damage counters on the Attacking Pokémon. Power Stomp — 200; discard
//! 2 Energy from this Pokémon.
//!
//! Twinleaf: Repulsor Axe reduces a RetaliateOnDamageDuringOpponentsNextTurn
//! Effect (`{ damage: 80 }`, target = the attacker's slot) arming
//! `retaliateOnDamageNextTurnPending` on the attacker's Active; Power Stomp is
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2) (ruling 1652: Energy units, never more cards than 2; no prompt
//! without Energy on the Active). The revenge itself
//! lives in the core AfterDamage reducer (see `attack.rs`).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronBoulderexTEFPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(80) })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::Units(2) })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
