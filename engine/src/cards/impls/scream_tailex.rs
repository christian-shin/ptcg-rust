//! Scream Tail ex (TWM): Scream — you can use this attack only if you go second, and only during your first turn. Your
//! opponent can't play any Supporter cards from their hand during their next turn. Crunch — 120; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Scream is usable only on turn 2 (a printed condition of the attack); it arms a lasting lock on the opponent over PlayTrainer
//! (a Supporter played from the hand). Crunch's Discard runs after the damage (a LeavePlay of the chosen Energy caused by the
//! attack; no cancel).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScreamTailex",
    attacks: &[
        AttackSpec {
            index: 0,
            // Only if you go second, during your first turn.
            steps: &[
                Step::before_damage(Op::Fail(FailSpec { unless: Cond::Not(&Cond::Cmp(Num::Turn, CmpOp::Ne, Num::Lit(2))), error: "CANNOT_USE_ATTACK" })),
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(PLAY_SUPPORTER_FROM_HAND, "BLOCKED_BY_EFFECT"))) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
