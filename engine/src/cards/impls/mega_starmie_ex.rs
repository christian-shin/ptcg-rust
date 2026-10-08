//! Mega Starmie ex (POR / M3): Jetting Blow — 120; also 50 damage to 1 of
//! your opponent's Benched Pokémon (PutDamageEffect, no Weakness). Nebula
//! Beam — 210; not affected by Weakness, Resistance, or effects.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaStarmieex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(50),
                target_damage_mul: 0,
                calc: DamageCalc::Put,
                when: Cond::True,
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
