//! Wash Rotom (DRI 61): Manual Wash - 20; heal 10 damage from each of your Pokémon. Gadget Show - 30x; this attack does 30 damage
//! for each Pokémon Tool attached to all of your Pokémon.
//!
//! Manual Wash: one RemoveCounters event per Pokémon of yours after the damage (Cresselia SFA 21). Gadget Show: 30 per Tool on all
//! your Pokémon, set before the damage (Heat Rotom DRI 43).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WashRotom",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Me),
                body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(10), clear_conditions: false }))],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::ToolCount(SlotSel::Pokemon(Who::Me)), &Num::Lit(30))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
