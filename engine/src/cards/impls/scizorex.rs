//! Scizor ex (TEF 111): Steel Wing — 70; during your opponent's next turn
//! this Pokémon takes 50 less damage from attacks. Cross Breaker — 120x;
//! discard up to 2 [M] Energy from this Pokémon, 120 damage for each card
//! discarded.
//!
//! Twinleaf: Steel Wing sets `player.active.damageReductionNextTurn = 50`.
//! Cross Breaker opens a DiscardEnergyPrompt on the Active (Energy cards, the
//! ones that don't provide [M] blocked, min 0, max 2, no cancel) even when nothing matches; an empty
//! answer sets the damage to 0, otherwise each transfer is a MOVE_CARDS to
//! the discard (no DiscardCardsEffect) and the damage is 120 x transfers.
//! R7A (ruling 1874): the Energy is chosen first, the damage is done, then the Energy is discarded (`move_cards_after_damage`).
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
