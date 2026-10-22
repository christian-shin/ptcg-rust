//! Miltank (MEG 106): Bellyful of Milk - flip 2 coins. If both of them are heads, heal all damage from 1 of your Pokémon. Tackle - 60.
//!
//! The two coins always flip (a printed flip); with 2 heads you choose 1 of your Pokémon and the heal removes all its damage
//! (a RemoveCounters event; the Pokémon may be undamaged: nothing happens).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miltank",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(2)),
            then: &[Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::Heads, CmpOp::Eq, Num::Lit(2)),
                yes: &[
                    Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me) }, purpose: PromptPurpose::HealTarget })),
                    Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::DamageOn(SlotExpr::Picked), clear_conditions: false })),
                ],
                no: &[],
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
