//! Iono's Kilowattrel (ASC 72): Flashing Draw - you must discard a Basic [L] Energy from this Pokémon in order to use this
//! Ability; once during your turn, you may draw cards until you have 6 cards in your hand. Mach Bolt - 70.
//!
//! The Energy is the cost (`Step::cost`, APR E-30): a LeavePlay (D1) caused by this Ability, and the Ability can't be used without
//! one on this Pokémon. Usability by the rules (D10, id2108): it can't be used with no cards in the deck or with 6 or more cards in
//! hand, as the draw would change nothing. The once-per-turn usage record ends with the owner's turn. The draw is one Draw
//! event of the cards missing from 6.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IonosKilowattrel",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy))), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::HasCard(Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")])))],
        steps: &[
            Step::cost(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::CostOne { pred: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]) }, ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::Last, CmpOp::Gt, Num::Lit(0)), yes: &[Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy))), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
