//! Arboliva ex (DRI): Oil Salvo — choose 1 of your opponent's Pokémon 6
//! times; 20 damage each time, not affected by Weakness or Resistance.
//! Aroma Shot — 160; this Pokémon recovers from all Special Conditions.
//!
//! Rule: each chosen Pokémon takes one Damage event (cause: this attack) per
//! 20; the attack flags NoWeakness / NoResistance keep Weakness and Resistance off
//! the Active Pokémon's share (APR B-08). A Pokémon "prevent all damage" protects
//! is still chosen and takes nothing (step 6, APR C-16). Aroma Shot removes all
//! five Special Conditions from this Pokémon (RemoveCondition events, cause: its
//! own attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Arbolivaex",
    attacks: &[
        // Oil Salvo: choose 1 of your opponent's Pokémon 6 times; 20 damage each time, not
        // affected by Weakness or Resistance.
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
                Step::after_damage(Op::SpreadDamage(SpreadDamageSpec {
                    chooser: Who::Me,
                    side: Who::Opp,
                    slots: SpreadSlots::Pokemon,
                    total_hp: 120,
                    unit_hp: 20,
                    cap_bonus_hp: Some(120),
                    apply: SpreadApply::Damage,
                })),
            ],
        },
        // Aroma Shot: this Pokémon recovers from all Special Conditions.
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Conditions(ConditionsSpec {
                target: MY_ACTIVE,
                change: ConditionChange::RemoveAll,
                gate: Gate::None,
                when: Cond::True,
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
