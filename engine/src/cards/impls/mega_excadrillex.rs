//! Mega Excadrill ex (PBL / M5): Undermine — 90, discard the top 2 cards of
//! your opponent's deck. Maximum Drilling — 200+, 130 more if this Pokémon
//! has at least 2 extra Energy attached.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaExcadrillex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            // At least 2 Energy more than the attack's cost.
            steps: &[Step::before_damage(more_damage_if(
                130,
                Cond::Cmp(Num::Sub(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::CostNow), CmpOp::Ge, Num::Lit(2)),
            ))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
