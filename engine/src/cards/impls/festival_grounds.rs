//! Festival Grounds (TWM 149 / PRE 108, Stadium): each Pokémon with any Energy attached recovers from all Special Conditions
//! and can't be affected by any.
//!
//! Two declarations over `SlotPred::HasEnergy`: `Modifier::Recover` (the table check removes the conditions they have) and a
//! `Modifier::Prevent` over GainCondition (any cause). A card that adds a condition (Dangerous Laser, an Ability) is still
//! playable or usable: its effect is just blocked (id290, JP FAQ Dangerous Laser). The Stadium can't be used. Festival Lead's
//! second attack lives on the Festival Lead Pokémon (`Modifier::AttackTwice`), which reads this Stadium.
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
