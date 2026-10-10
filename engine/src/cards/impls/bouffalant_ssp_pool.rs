//! Bouffalant (SSP 151): Ready to Ram — 40; during your opponent's next turn, if this Pokémon is damaged by an attack
//! (even if Knocked Out), put 6 damage counters on the Attacking Pokémon. Smashing Headbutt — 150; discard 2 Energy
//! from this Pokémon.
//!
//! Ready to Ram arms `Lasting::Retaliate(60)` (one ApplyEffect event on this Pokémon: "prevent all effects of attacks"
//! stops it, id2341). When a Damage event from an opponent's attack hits the Pokémon the effect lasts on, a step 7 trigger
//! puts the counters on the Attacking Pokémon, wherever it is, if it is still in play: a PlaceCounters event caused by
//! Bouffalant's own Ready to Ram attack (id2408, id1958), so Mist Energy on the Attacking Pokémon refuses it. It runs
//! even if the damage Knocks this Pokémon Out (the Knock Out is taken at the state check, after the trigger).
//! Smashing Headbutt discards 2 Energy units, never more cards than 2; no prompt without Energy on the Active.
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
