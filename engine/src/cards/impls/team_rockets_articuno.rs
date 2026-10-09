//! Team Rocket's Articuno (DRI): Repelling Veil — prevent all effects of the
//! opponent's attacks done to your Basic Team Rocket's Pokémon. Dark Frost —
//! 60+; 60 more if this Pokémon has Team Rocket's Energy attached.
//!
//! Twinleaf (fixed in phase 4b, R1-2): Repelling Veil is Mist Energy's
//! pattern for the owner's Basic Team Rocket's Pokémon: every
//! AbstractAttackEffect whose target slot holds a Basic Team Rocket's Pokémon
//! is prevented, unless it is ApplyWeakness / PutDamage / DealDamage, when
//! this Articuno is in play on the target owner's side, its Ability isn't
//! blocked (probe for the owner), the effect comes from the owner's
//! opponent's Pokémon and the source slot holds a Pokémon. (It used to
//! prevent only PutCountersEffect, with no Ability-lock check.) Dark Frost
//! looks for an Energy named "Team Rocket's Energy" on the attacker's Active
//! (fixed in phase 4b: it compared against "Team Rocket Energy", so the bonus
//! never applied).
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
