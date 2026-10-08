//! Survival Brace (TWM, ACE SPEC tool): if the Pokémon this card is attached
//! to has full HP and would be Knocked Out by damage from an opponent's
//! attack, that Pokémon is not Knocked Out and its remaining HP becomes 10
//! instead. Then, discard this card.
//!
//! Twinleaf: on a PutDamageEffect whose target holds this tool, during the
//! attack phase and not from the owner's own Pokémon (phase 4b: it used to
//! trigger on any PutDamageEffect), unless the tool is blocked, when the slot
//! has no damage and `effect.damage >=` its HP (CheckHpEffect by the owner):
//! sets `surviveOnTenHPReason` and discards the tool from every slot of the
//! owner holding it. The core then caps the damage at HP - 10 whenever it
//! reached HP (phase 4b: exactly lethal damage used to Knock Out anyway).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SurvivalCast",
    passives: &[Passive { origin: RuleSource::Tool, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec {}) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
