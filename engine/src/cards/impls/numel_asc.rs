//! Numel (ASC 27): Incandescent Body - if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's Pokémon (even if this Pokémon is Knocked Out), the Attacking Pokémon is now Burned. Combustion - 20.
//!
//! Incandescent Body is a trigger over the Damage event of an opponent's attack on the Active Pokémon (it fires for a Knock
//! Out too); a GainCondition(Burned) on the Attacking Pokémon caused by the Ability (Heatran TWM 123).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NumelASC",
    triggers: &[
        Trigger {
            origin: RuleSource::Ability,
            event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
            steps: &[Step::new(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::Slot(SlotExpr::CausePokemon, SlotPred::IsActive)]),
                yes: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::CausePokemon, change: ConditionChange::Add(&[SpecialCondition::Burned]), when: Cond::True }))],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
