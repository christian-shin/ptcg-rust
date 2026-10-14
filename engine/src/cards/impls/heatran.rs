//! Heatran (TWM 123): Incandescent Body — if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if this Pokémon is Knocked Out), the Attacking Pokémon is now Burned. Steel Burst — 50×; discard all [M]
//! Energy from this Pokémon. This attack does 50 damage for each card you discarded in this way.
//!
//! Incandescent Body is an `Event::On` trigger over the Damage event of an opponent's attack (the damaged Pokémon is Active:
//! `DamagedActive`; it fires even for a Knock Out, and the Attacking Pokémon, `CausePokemon`, is read where it is then): a GainCondition(Burned) on the Attacker caused by this Ability, so Hide 'n'
//! Sneak and the other Ability immunities stop it. Steel Burst counts the [M] Energy cards on this Pokémon,
//! adds 50 per card before the damage and discards those cards after it.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Heatran",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Sub(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::MatchingEntries(ct::METAL)), &Num::Lit(1)), &Num::Lit(50)), when: Cond::True })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::Provides(ct::METAL), ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::Slot(SlotExpr::CausePokemon, SlotPred::IsActive)]), yes: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::CausePokemon, change: ConditionChange::Add(&[SpecialCondition::Burned]), when: Cond::True }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
