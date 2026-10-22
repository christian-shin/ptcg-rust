//! Glastrier (ASC 54): Ice Shot - 20; this attack also does 20 damage to 1 of your opponent's Benched Pokémon. Frosty Typhoon - 130; during your next turn, this Pokémon can't use Frosty Typhoon.
//!
//! Ice Shot's 20 is the attack's damage to a chosen Benched Pokémon (one Damage event, `DamageCalc::Put`: no Weakness or Resistance; Marnie's Grimmsnarl ex). Frosty Typhoon arms a lasting lock naming this attack on this Pokémon until the end of your next turn (Miraidon ex TEF 122).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Glastrier",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp) }),
                hp: Num::Lit(20),
                target_damage_mul: 0,
                calc: DamageCalc::Put,
                when: Cond::True,
            })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
