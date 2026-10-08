//! Luxray ex (TWM): Piercing Gaze — 120; look at your opponent's hand and
//! discard 1 card you find there. Volt Strike — 250; discard all Energy from
//! this Pokémon.
//!
//! Twinleaf: nothing on an empty opponent's hand, else a ChooseCardsPrompt
//! (CHOOSE_CARD_TO_DECK, min 1 max 1, no cancel) on it; a pick is
//! MOVE_CARDS'd to their discard, followed by a no-op
//! MOVE_CARDS(player.supporter → player.discard, [this]) (quirk kept).
//! Volt Strike: CheckProvidedEnergyEffect on the Active → DiscardCardsEffect.
//!
//! Fixed (phase 4b, W4): the prompt was min 0, so the attacker could discard
//! nothing; the card says "Discard a card you find there".
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Luxrayex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT }))] }, AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::AllProvided }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
