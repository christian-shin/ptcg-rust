//! Sigilyph (WHT 38): Reflect — during your opponent's next turn, this Pokémon takes 40 less damage from attacks (after applying
//! Weakness and Resistance). Telekinesis — this attack does 70 damage to 1 of your opponent's Pokémon. This attack's damage isn't
//! affected by Weakness or Resistance.
//!
//! Reflect arms the lasting "takes less damage". Telekinesis sets the attack's no-Weakness and no-Resistance flags before the damage
//! and does 70 to the chosen Pokémon (chosen at step D), the Active Pokémon or a Benched one: one Damage event caused by the attack,
//! effects on the damaged Pokémon still apply (only Weakness and Resistance are ignored).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SigilyphWHT",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(40, DamageSource::Any) }))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                    hp: Num::Lit(70),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
