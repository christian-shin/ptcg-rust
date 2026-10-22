//! Electivire ex (DRI 69): Dual Bolt - this attack does 50 damage to 2 of your opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.) High-Voltage Press - 180+; if this Pokémon has at least 2 extra Energy attached (in addition to this attack's cost), this attack does 100 more damage.
//!
//! Dual Bolt is Steven's Skarmory DRI 142's Sonic Double (exactly min(2, the opponent's Pokémon in play) targets, `DamageCalc::Auto`); High-Voltage Press is Mega Excadrill ex's Maximum Drilling (Energy provided minus the attack's cost, at least 2) with 100 more.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ElectivireEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), }), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(50), ..EachSlotSpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(100, Cond::Cmp(Num::Sub(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::CostNow), CmpOp::Ge, Num::Lit(2)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
