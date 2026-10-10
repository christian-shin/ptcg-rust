//! Deoxys (M4 32): Psyspear - 120; if this Pokémon has at least 2 extra
//! Energy attached, also 120 damage to 1 of your opponent's Benched Pokémon.
//!
//! With 2 or more Energy beyond the attack's printed cost, one Damage event on the chosen Benched Pokémon (a mandatory
//! pick, only when the opponent has a Benched Pokémon; no Weakness or Resistance), with the attack as its cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Deoxys2",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(120), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::Cmp(Num::Sub(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::PrintedCost), CmpOp::Ge, Num::Lit(2)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
