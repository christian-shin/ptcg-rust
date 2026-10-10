//! Survival Brace (TWM, ACE SPEC tool): if the Pokémon this card is attached
//! to has full HP and would be Knocked Out by damage from an opponent's
//! attack, that Pokémon is not Knocked Out and its remaining HP becomes 10
//! instead. Then, discard this card.
//!
//! A survive-on-10 replacement in the damage calculation (`damage::survive_on_10`), for the damage of an opponent's
//! attack only (not the owner's own Pokémon's attack): when the Pokémon has no damage and the Damage event's amount reaches
//! its HP (exactly lethal included), the damage is capped at HP - 10 and this Tool is discarded, unless its lock is on.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SurvivalCast",
    passives: &[Passive { origin: RuleSource::Tool, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::ToolIfFullHp }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
