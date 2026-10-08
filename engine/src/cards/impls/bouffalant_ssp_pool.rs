//! Bouffalant (SSP 151): Ready to Ram — 40; during your opponent's next turn,
//! if this Pokémon is damaged by an attack (even if Knocked Out), put 6 damage
//! counters on the Attacking Pokémon. Smashing Headbutt — 150; discard 2
//! Energy from this Pokémon.
//!
//! Twinleaf: Ready to Ram reduces a RetaliateOnDamageDuringOpponentsNextTurn
//! Effect (`{ damage: 60 }`); Smashing Headbutt is
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2) (ruling 1652: Energy units, never more cards than 2; no prompt
//! without Energy on the Active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BouffalantSSPPool",
    attacks: &[
        // Ready to Ram: during your opponent's next turn, if this Pokémon is damaged by an attack
        // (even if Knocked Out), put 6 damage counters on the Attacking Pokémon.
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(60) }))] },
        // Smashing Headbutt: discard 2 Energy from this Pokémon.
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
