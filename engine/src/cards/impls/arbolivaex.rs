//! Arboliva ex (DRI): Oil Salvo — choose 1 of your opponent's Pokémon 6
//! times; 20 damage each time, not affected by Weakness or Resistance.
//! Aroma Shot — 160; this Pokémon recovers from all Special Conditions.
//!
//! Twinleaf: Oil Salvo is a non-cancellable PutDamagePrompt (120 damage in
//! multiples of 20, per-target cap = printed HP + 120), then
//! DAMAGE_OPPONENT_POKEMON per entry, so the Active's share goes through a
//! DealDamageEffect. Fixed (R1-1): the attack sets `ignoreWeakness` and
//! `ignoreResistance` (the damage isn't affected by Weakness or Resistance),
//! and Aroma Shot removes all five Special Conditions from the attacker's
//! Active (RemoveSpecialConditionsEffect(effect, undefined); it used to have
//! no handler).
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
                cause: Cause::Direct,
                gate: Gate::None,
                when: Cond::True,
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
