//! Ceruledge (SSP 35): Cursed Edge: discard all Special Energy from each of your opponent's Pokémon. Black Blaze Slash:
//! 160; during your next turn, this Pokémon can't attack.
//!
//! Cursed Edge discards Pokémon by Pokémon; each Special Energy leaving is a LeavePlay event with this attack as its cause,
//! so Mist Energy or Hide 'n' Sneak on a Pokémon refuses it for that Pokémon only (id2222) while the others lose theirs; a
//! card stays a Special Energy card while its effects are off (id2085). Black Blaze Slash arms a lasting lock over this
//! Pokémon's attacks (`CANT_ATTACK`, until the end of your next turn).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ceruledge@SSP",
    attacks: &[
        // Blaze Curse: discard all Special Energy from each of your opponent's Pokémon.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Opp),
                body: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::Special, ..DiscardEnergySpec::DEFAULT }))],
            }))],
        },
        // Amethyst Rage: during your next turn, this Pokémon can't attack.
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
