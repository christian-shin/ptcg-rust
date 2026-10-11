//! Luxray ex (TWM 68): Piercing Gaze - 120; your opponent reveals their hand; discard a card you find there. Volt Strike -
//! 250; discard all Energy from this Pokémon.
//!
//! Piercing Gaze: the attacker looks at the hand and chooses 1 card (one is required: "a card"), which is one Discard
//! event from the opponent's hand (cause: this attack); an empty hand does nothing. Volt Strike discards every Energy card
//! on the Pokémon after the damage: a LeavePlay per card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Luxrayex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))] }, AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
