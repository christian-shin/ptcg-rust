//! Paldean Tauros (SSP): Spirited Tackle - 90+, 90 more damage if the
//! opponent's Active Pokémon is a Stage 1 Pokémon.
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "PaldeanTauros",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(90), when: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::Stage(Stage::Stage1 as u8))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
