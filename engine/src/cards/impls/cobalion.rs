//! Cobalion (BLK 66): Righteous Edge - 20; discard a Special Energy from your opponent's Active Pokémon. Metal Arms - 80+; if this
//! Pokémon has a Pokémon Tool attached, this attack does 40 more damage.
//!
//! Righteous Edge: the attacker chooses 1 Special Energy attached to the Defending Pokémon and it is discarded after the damage
//! (Farfetch'd's Mach Cut). Metal Arms reads the Tools attached to this Pokémon before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cobalion",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::Chosen { pred: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]) },
                ..DiscardEnergySpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(40, Cond::Slot(SlotExpr::This, SlotPred::AnyTool(Pred::Any))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
