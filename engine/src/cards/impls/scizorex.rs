//! Scizor ex (TEF 111): Steel Wing — 70; during your opponent's next turn
//! this Pokémon takes 50 less damage from attacks. Cross Breaker — 120x;
//! discard up to 2 [M] Energy from this Pokémon, 120 damage for each card
//! discarded.
//!
//! Steel Wing is an ApplyEffect event on this Pokémon (`Lasting::TakesLessDamage(50)`), read in the damage calculation
//! after Weakness and Resistance until the end of the opponent's next turn. Cross Breaker asks for the Energy first
//! (0 to 2; the prompt appears even with none), the damage is then 120 for each chosen (ruling 1874), and the Energy is
//! discarded after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Scizorex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(50) }))] },
        AttackSpec {
            index: 1,
            // Discard up to 2 [M] Energy from this Pokémon: 120 damage for each card discarded.
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Scoped { scope: PromptScope::Active, min: Num::Lit(0), max: Num::Lit(2), kind: EnergyKind::Provides(ct::METAL), clamp: false }, into: Some(0), ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), op: DamageOp::Set, per: 120 })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
