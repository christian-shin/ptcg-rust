//! Festival Grounds (TWM, stadium): each Pokémon with any Energy attached
//! recovers from all Special Conditions and can't be affected by any.
//!
//! Effects that would add a Special Condition are prevented, and the table
//! check sweeps the ones a Pokémon has. A condition-adding Ability or card is
//! still usable: its effect is just blocked (ruling 290). The stadium
//! can't be "used". Festival Lead's double attack lives on the Festival Lead
//! cards (runtime `barrage`, see Dipplin TWM) and the core useAttack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FestivalGrounds",
    passives: &[
        // Each Pokémon with any Energy attached recovers from all Special Conditions and can't be
        // affected by any: effects that add them are prevented, and the ones it has are swept.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::ConditionImmunity(ConditionImmunitySpec { conds: &[], subject: SlotPred::HasEnergy, prevent: true, sweep: true }),
        },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
