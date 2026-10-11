//! Goldeen (TWM): Festival Lead. Whirlpool - 10; flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//!
//! Festival Lead (with Festival Grounds in play, use an attack twice) is a separate Ability; Whirlpool only clears its attack
//! flag before the damage. The coin is flipped only when the
//! Defending Pokémon has an Energy; on heads the discard is a LeavePlay (to the discard pile) of the Energy the attacker chooses,
//! caused by this attack (D1).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Goldeen@TWM|PRE",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::FestivalLead, value: false })),
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::AnySlot(SlotSel::One(SlotExpr::Active(Who::Opp)), SlotPred::HasEnergy), heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
