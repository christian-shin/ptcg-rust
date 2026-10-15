//! Luxray ex (TWM 68): Piercing Gaze - 120; your opponent reveals their hand; discard a card you find there. Volt Strike -
//! 250; discard all Energy from this Pokémon.
//!
//! Piercing Gaze: after the damage the opponent's hand is revealed to the attacker (a Reveal, also when the hand is empty:
//! the text says "reveals their hand", as Hoothoot TEF and Mega Absol ex), then the attacker chooses 1 card (one is
//! required: "a card"), which is one Discard event from the opponent's hand (cause: this attack); an empty hand does
//! nothing. Volt Strike discards every Energy card on the Pokémon after the damage: a LeavePlay per card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LuxrayEx",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })),
            Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))] }, AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
