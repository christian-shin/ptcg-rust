//! Heatran (TWM 123): Incandescent Body — if this Pokémon is in the Active
//! Spot and is damaged by an attack from your opponent's Pokémon (even if it
//! is Knocked Out), the Attacking Pokémon is now Burned. Steel Burst — 50x;
//! discard all [M] Energy from this Pokémon, 50 damage for each card
//! discarded.
//!
//! Twinleaf: Steel Burst discards the cards of the Active's
//! CheckProvidedEnergy map whose entry provides [M] in one DiscardCardsEffect
//! and adds `(listed - 1) * 50` (counted before the discard resolves).
//! Fixed (phase 4b, R3): it used to discard and count every attached Energy
//! card, whatever its type. Incandescent Body reacts to AfterDamageEffect on
//! any list holding this card: the lock probe runs for the *attacking*
//! player, and the burn is a direct `source.addSpecialCondition` during the
//! attack phase.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Heatran",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Sub(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::ProvidedCardsOf(ct::METAL)), &Num::Lit(1)), &Num::Lit(50)), when: Cond::True })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::AllProvidedOf(ct::METAL) })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: true, removes_attacker_energy: false }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AttackerInPlay, Cond::Slot(SlotExpr::Attacker, SlotPred::IsActive)]), yes: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Attacker, change: ConditionChange::Add(&[SpecialCondition::Burned]), cause: Cause::Direct, gate: Gate::None, when: Cond::True }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
