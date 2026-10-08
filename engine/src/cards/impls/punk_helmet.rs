//! Punk Helmet (PFL, tool): if the [D] Pokémon this card is attached to is in
//! the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if this Pokémon is Knocked Out), place 4 damage counters on the
//! Attacking Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PunkHelmet",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::OnDamagedByAttack(OnDamagedByAttackSpec::DEFAULT),
        steps: &[Step::new(Op::If(IfSpec {
            // 4 damage counters on the Attacking Pokémon, if the holder is a [D] Pokémon.
            cond: Cond::Slot(SlotExpr::This, SlotPred::TypeIs(crate::types::ct::DARK)),
            yes: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(40), target_damage_mul: 0, calc: DamageCalc::Direct, when: Cond::True }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
