//! Zarude (PBL 56): Overhead Throw - 30; this attack also does 30 damage to 1 of your Benched Pokémon. (Don't apply Weakness and
//! Resistance for Benched Pokémon.) Shadowy Whip - 100+; if your Benched Pokémon have any Shadowy [D] Energy attached, this attack
//! does 70 more damage.
//!
//! Overhead Throw: the attacker chooses 1 of their own Benched Pokémon (none: nothing more happens) at step D; one Damage event of
//! 30 caused by the attack, without Weakness or Resistance (APR B-08). Shadowy Whip adds 70 before the damage when any Benched
//! Pokémon of yours has a card named Shadowy Darkness Energy attached.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZarudePBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me) }),
                hp: Num::Lit(30),
                target_damage_mul: 0,
                calc: DamageCalc::Put,
                when: Cond::True,
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(more_damage_if(70, Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::HasEnergyNamed("Shadowy Darkness Energy"))))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
