//! Gravity Mountain (SSP 177, Stadium): each Stage 2 Pokémon in play (both yours and your opponent's) gets -30 HP.
//!
//! One `Modifier::HpMod` (-30, `SlotPred::StageIs(Stage 2)`) while this is the Stadium in play. The HP change ends with the
//! Stadium, and a Pokémon whose damage then reaches its HP is Knocked Out at the state check (id2305). The Stadium can't be
//! used.
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "GravityMountain",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::HpMod(HpModSpec { amount: -30, subject: SlotPred::StageIs(Stage::Stage2), guard: Cond::True }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
