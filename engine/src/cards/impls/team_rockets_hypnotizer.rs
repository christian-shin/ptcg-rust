//! Team Rocket's Hypnotizer (ASC 206, Pokémon Tool): if the Team Rocket's Pokémon this card is attached to is in the Active Spot
//! and is damaged by an attack from your opponent's Pokémon (even if this Team Rocket's Pokémon is Knocked Out), the Attacking
//! Pokémon is now Asleep.
//!
//! A trigger over the Damage event of an opponent's attack on the Pokémon holding the Tool while it is the Active Pokémon
//! (`DamagedActive`, id1992), resolved in the E-03 window as Deluxe Bomb: the holder must carry the "Team Rocket's" name tag; the
//! Attacking Pokémon, wherever it is now, gets one GainCondition(Asleep) caused by the Tool (nothing when it left play; a
//! Pokémon that can't be Asleep stays as it is). The Tool is not discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsHypnotizer",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(crate::types::tag::TEAM_ROCKET)])), EventPred::DamagedActive])),
        steps: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::CausePokemon, change: ConditionChange::Add(&[SpecialCondition::Asleep]), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
