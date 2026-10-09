//! Yveltal (SFA 35): Corrosive Winds — put 2 damage counters on each of your
//! opponent's Pokémon that has any damage counters. Destructive Beam — 100;
//! flip a coin, if heads discard an Energy from the opponent's Active.
//!
//! Twinleaf: no coin flip when the Active has no Energy card; the discard is
//! a ChooseCardsPrompt on the Active then a DiscardCardsEffect.
//!
//! Fixed (phase 4b, W4): printed data only, Resistance is Fighting -30 (was -20).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yveltal@SFA",
    attacks: &[
        AttackSpec {
            index: 0,
            // Corrosive Winds: 2 damage counters on each of your opponent's Pokémon that has any damage counters.
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Counters, amount: Num::Lit(2), only_damaged: true, ..EachSlotSpec::DEFAULT }))],
        },
        AttackSpec {
            index: 1,
            // Destructive Beam: flip a coin; if heads, discard an Energy from the opponent's Active Pokémon (no flip without one).
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Slot(OPP_ACTIVE, SlotPred::HasEnergy),
                yes: &[Step::new(Op::Coin(CoinSpec {
                    heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
                    ..CoinSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
