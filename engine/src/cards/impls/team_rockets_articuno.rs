//! Team Rocket's Articuno (DRI): Repelling Veil — prevent all effects of the opponent's attacks done to your Basic Team
//! Rocket's Pokémon. Dark Frost — 60+; 60 more if this Pokémon has Team Rocket's Energy attached.
//!
//! Repelling Veil is one `Prevent` naming no kind (`EFFECTS_OF_OPP_ATTACKS`) on each of your Basic Team Rocket's
//! Pokémon: it ranges over every event with an effect the opponent's attacks cause to them (counters placed or moved
//! onto them, Special Conditions, switches, lasting effects: APR C-04 / C-05 / C-17, id2025, id2155, id2150), never over
//! Damage. Existing effects are not removed (id2341). It needs this Articuno in play with its Ability working.
//! Dark Frost looks for an Energy named "Team Rocket's Energy" on this Pokémon.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsArticuno",
    // Repelling Veil: prevent all effects of the opponent's attacks done to your Basic Team Rocket's Pokémon.
    passives: &[
        // Every event the opponent's attacks cause to them, the switches included (APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::OnMySide, SlotPred::Top(Pred::All(&[Pred::Basic, Pred::Tag(tag::TEAM_ROCKET)]))]), EFFECTS_OF_OPP_ATTACKS)) },
    ],
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(60, Cond::Slot(MY_ACTIVE, SlotPred::HasEnergyNamed("Team Rocket's Energy"))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
