//! Stonjourner (MEG 81): Stony Kick - 20; this attack also does 20 damage to 1 of your opponent's Benched Pokémon. Boundless Power - 140; during your next turn, this Pokémon can't use attacks.
//!
//! Stony Kick's 20 is the attack's damage to a chosen Benched Pokémon (one Damage event, `DamageCalc::Put`: no Weakness or
//! Resistance; Glastrier ASC 54). Boundless Power arms a lasting `CANT_ATTACK` lock on this Pokémon until the end of your
//! next turn (Bouffalant SCR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StonjournerMEG",
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
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
