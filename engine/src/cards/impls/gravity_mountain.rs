//! Gravity Mountain (SSP, stadium): each Stage 2 Pokémon in play (both
//! yours and your opponent's) gets -30 HP.
//!
//! Twinleaf: CheckHpEffect while this is the stadium in play, unless the
//! stadium effect is blocked for the target's owner; the stadium can't be used.
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "GravityMountain",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::HpMod(HpModSpec { amount: -30, subject: SlotPred::StageIs(Stage::Stage2), guard: Cond::True }),
        },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
