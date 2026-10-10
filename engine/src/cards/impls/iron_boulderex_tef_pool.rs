//! Iron Boulder ex (TEF 99): Repulsor Axe — 60; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if it is Knocked Out),
//! put 8 damage counters on the Attacking Pokémon. Power Stomp — 200; discard
//! 2 Energy from this Pokémon.
//!
//! Repulsor Axe arms `Lasting::Retaliate(80)` (an ApplyEffect event on this Pokémon). During the opponent's next turn
//! each Damage event an attack does to it, even one that Knocks it Out, records the trap; after that attack's damage the
//! 8 counters are a PlaceCounters event with Repulsor Axe as its cause on the Attacking Pokémon wherever it is now
//! (id534, id2371; nothing when it left play, id588), so Mist Energy or Hide 'n' Sneak on it refuses them (id2408,
//! id1958). Power Stomp discards 2 Energy units, never more cards than 2 (id2127).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronBoulderexTEFPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(80) })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
