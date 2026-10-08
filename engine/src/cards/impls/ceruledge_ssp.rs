//! Ceruledge (SSP): Blaze Curse — discard all Special Energy from each of your
//! opponent's Pokémon. Amethyst Rage — 160; during your next turn, this
//! Pokémon can't attack.
//!
//! Twinleaf: for the opponent's Active, then each Bench slot, one MOVE_CARDS
//! (no source card) of the attached Special Energy to their discard pile.
//! R7A (ruling 1620 and the attack flow chart): the Special Energy is discarded after the damage (`move_cards_after_damage`).
//!
//! Fixed (phase 4b, R7F-17; rulings 1843, 1724): the discard ignored Mist
//! Energy; each Pokémon is first probed with a DiscardCardsEffect without
//! cards (the effect of the attack on that Pokémon), and a prevented one keeps
//! its Special Energy.
//! R7A + R7F: the probe comes first, then the deferred discard of the Energy of the Pokémon that is not protected.
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
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
