//! Festival Grounds (TWM, stadium): each Pokémon with any Energy attached
//! recovers from all Special Conditions and can't be affected by any.
//!
//! Events batch 4: a `Prevent` over GainCondition (any cause) on those
//! Pokémon, and `Recover`: the table check makes them recover from the ones
//! they have. A condition-adding Ability or card is
//! still usable: its effect is just blocked (ruling 290). The stadium
//! can't be "used". Festival Lead's double attack lives on the Festival Lead
//! cards (runtime `barrage`, see Dipplin TWM) and the core useAttack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FestivalGrounds",
    passives: &[
        // Each Pokémon with any Energy attached recovers from all Special Conditions and can't be
        // affected by any.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Recover(RecoverSpec { conds: &[], subject: SlotPred::HasEnergy }) },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::HasEnergy, EventPred::Kind(EventKind::GainCondition))) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
