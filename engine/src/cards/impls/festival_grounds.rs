//! Festival Grounds (TWM, stadium): each Pokémon with any Energy attached
//! recovers from all Special Conditions and can't be affected by any.
//!
//! Twinleaf: implemented as a CheckTableStateEffect sweep (conditions are
//! cleared whenever the table state is checked, not prevented). The stadium
//! can't be "used". Festival Lead's double attack lives on the Festival Lead
//! cards (runtime `barrage`, see Dipplin TWM) and the core useAttack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FestivalGrounds",
    passives: &[
        // Each Pokémon with any Energy attached recovers from all Special Conditions. Today's
        // behavior kept (B-PC-38): swept at the table check, not prevented.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::ConditionImmunity(ConditionImmunitySpec { conds: &[], subject: SlotPred::HasEnergy, prevent: false, sweep: true }),
        },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
