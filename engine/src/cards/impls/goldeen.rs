//! Goldeen (TWM): Festival Lead. Whirlpool — 10; flip a coin, if heads
//! discard an Energy from the opponent's Active Pokémon.
//!
//! Twinleaf: Festival Lead's runtime `barrage` flag is written on every use
//! (fixed in phase 4b, R4: it used to be skipped when the opponent's Active had
//! no Energy card, and left unchanged while the Ability was blocked), before
//! the no-Energy return. The coin flip is still skipped without Energy.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Goldeen@Goldeen TWM|Goldeen PRE",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::FestivalLead, value: false })),
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::AnySlot(SlotSel::One(SlotExpr::Active(Who::Opp)), SlotPred::HasEnergy), heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
