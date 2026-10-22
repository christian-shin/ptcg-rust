//! Floatzel (DRI 58): Whirlpool - 30; discard an Energy from your opponent's Active Pokémon. Aqua Slash - 140; during your next turn, this Pokémon can't attack.
//!
//! One Energy chosen from the opponent's Active Pokémon after the damage (Goldeen); a lasting `CANT_ATTACK` lock on this Pokémon until the end of my next turn (Bouffalant SCR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Floatzel",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
