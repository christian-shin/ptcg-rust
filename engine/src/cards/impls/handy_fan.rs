//! Handheld Fan (TWM 150, Tool): if the Pokémon this card is attached to is in the Active Spot and is damaged by an
//! attack from your opponent's Pokémon (even if this Pokémon is Knocked Out), move an Energy from the Attacking Pokémon
//! to 1 of your opponent's Benched Pokémon.
//!
//! An `OnDamagedByAttack` trigger: the Damage event records it when the holder is the Active Pokémon then (id1992: the
//! Active Spot is read when the damage is done), and it resolves after the attack's own effects (step 7; id2091, id2124,
//! id2125, id2126), while the Attacking Pokémon is in play wherever it is then: one MoveEnergy event by the Tool onto
//! one of the attacking player's other Benched Pokémon, exactly 1 Energy (nothing when it has none or no other Benched
//! Pokémon). A Tool lock turns it off.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HandyFan",
    triggers: &[
        Trigger { origin: RuleSource::Tool, event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: false, removes_attacker_energy: true, attacker_required: true }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::ToolBlocked)]), yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(1), same_target: false, via_effect: false, kind: EnergyKind::Any }, to: EnergyDest::Stay, ..DiscardEnergySpec::DEFAULT }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
