//! Core Memory (POR 70, Pokémon Tool, Technical Machine): the Mega Zygarde ex this card is attached to can use the attack on this
//! card. (You still need the necessary Energy to use this attack.) Geobuster [F][F][F][F], 350: discard all Energy from this
//! Pokémon.
//!
//! The attack is the Tool's printed attack (the Active Pokémon's attack list starts with its Tool's attacks,
//! `engine::turn::printed_attack_list`: a Technical Machine's attack is used by the Pokémon, which pays its cost). "The Mega
//! Zygarde ex this card is attached to" is `CardSpec::attack_holder`: on any other Pokémon the attack isn't offered. Geobuster's
//! discard is its own effect after the damage: a LeavePlay (to the discard pile) of each Energy attached to the attacking
//! Pokémon, caused by this attack (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CoreMemory",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))],
    }],
    // The Mega Zygarde ex this card is attached to can use the attack on this card.
    attack_holder: Some(SlotPred::Named("Mega Zygarde ex")),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
