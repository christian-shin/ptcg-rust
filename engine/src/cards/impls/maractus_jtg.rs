//! Maractus (JTG): Explosive Needle — if this Pokémon is in the Active Spot
//! and is Knocked Out by damage from an attack from your opponent's Pokémon,
//! put 6 damage counters on the Attacking Pokémon. Corner — 20; the
//! Defending Pokémon can't retreat during your opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Maractus@JTG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnKnockOut(OnKnockOutSpec {}),
        // 6 damage counters on the Attacking Pokémon, written straight onto it.
        steps: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(60), target_damage_mul: 0, calc: DamageCalc::Direct, when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
