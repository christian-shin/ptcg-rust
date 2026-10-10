//! Shaymin (DRI): Flower Curtain — prevent all damage done to your Benched
//! Pokémon that don't have a Rule Box by attacks from your opponent's
//! Pokémon. Smash Kick — 30.
//!
//! One `Prevent` over `Kind(Damage)` (`DAMAGE_BY_OPP_ATTACKS`) on your Benched Pokémon without a Rule Box, read at step 6
//! of the damage calculation (APR C-16): any attack of the opponent's Pokémon, damage only (counters are not damage,
//! APR C-07). It is an Ability: a Shaymin whose Ability is off protects nothing. The Rule Box is read over the target's
//! cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shaymin",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Your Benched Pokémon that don't have a Rule Box.
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::OnMySide, SlotPred::IsBench, SlotPred::Not(&SlotPred::RuleBox)]), DAMAGE_BY_OPP_ATTACKS)),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
