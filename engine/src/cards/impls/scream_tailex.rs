//! Scream Tail ex (TWM): Scream — can only be used if you go second, during
//! your first turn; during your opponent's next turn, they can't play any
//! Supporter cards from their hand. Crunch — 120; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Twinleaf: Scream throws CANNOT_USE_ATTACK unless `state.turn === 2`. Crunch
//! opens a non-cancellable ChooseCardsPrompt on the Defending Pokémon (when it
//! has an Energy card) and reduces a DiscardCardsEffect in the callback.
//!
//! Fixed (phase 4b, R2): Crunch discarded in the attack handler, before the
//! damage (the Defending Pokémon's Spiky Energy was already gone); it now runs
//! in AfterAttackEffect with a fresh AttackEffect's data.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScreamTailex",
    attacks: &[
        AttackSpec {
            index: 0,
            // Only if you go second, during your first turn.
            steps: &[
                Step::before_damage(Op::Fail(FailSpec { unless: Cond::Not(&Cond::Cmp(Num::Turn, CmpOp::Ne, Num::Lit(2))), error: "CANNOT_USE_ATTACK" })),
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(Locked::Supporter) })),
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
