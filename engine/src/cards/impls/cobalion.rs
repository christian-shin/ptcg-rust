//! Cobalion (BLK 66): Righteous Edge - 20; discard a Special Energy from your opponent's Active Pokémon. Metal Arms - 80+; if
//! this Pokémon has a Pokémon Tool attached, this attack does 40 more damage.
//!
//! Righteous Edge discards one chosen Special Energy card of the Defending Pokémon after the damage (a LeavePlay caused by the
//! attack; Farfetch'd). Metal Arms adds 40 before the damage when a Tool is attached to this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cobalion",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::Chosen { pred: Pred::SpecialEnergy },
                ..DiscardEnergySpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(40, Cond::Cmp(Num::ToolCount(SlotSel::One(SlotExpr::This)), CmpOp::Gt, Num::Lit(0))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
