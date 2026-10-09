//! Lively Stadium ("Exciting Stadium SSP", stadium): each Basic Pokémon in
//! play (both yours and your opponent's) gets +30 HP.
//!
//! Twinleaf: CheckHpEffect while this is the stadium in play, unless the
//! stadium effect is blocked for the target's owner; the stadium can't be used.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ExcitingStadium",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        // Each Basic Pokémon in play (both yours and your opponent's) gets +30 HP.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::HpMod(HpModSpec { amount: 30, subject: SlotPred::Basic, guard: Cond::True }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
