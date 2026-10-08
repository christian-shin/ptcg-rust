//! Team Rocket's Mewtwo ex (DRI): Power Saver — can't attack unless you have
//! 4 or more Team Rocket's Pokémon in play. Erasure Ball — 160+; discard up
//! to 2 Energy from your Benched Pokémon, 60 more damage for each.
//!
//! Twinleaf: Power Saver reacts to any UseAttackEffect whose source slot
//! holds this card (after the Ability-lock check). Erasure Ball's prompt
//! allows any Energy (phase 4b: the filter used to be Basic only, but the text
//! says "Energy"), is skipped without a Benched Pokémon, and each chosen card
//! is a separate MOVE_CARDS to the discard pile.
//! R7A (ruling 1874): the Energy is chosen first, the damage is done, then the Energy is discarded (`move_cards_after_damage`).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMewtwoex",
    // Power Saver: this Pokémon can't attack unless you have 4 or more Team Rocket's Pokémon in play.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockAttack(BlockAttackSpec {
            on: AttackBlockOn::UseAttack,
            unless: Cond::Cmp(Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(tag::TEAM_ROCKET)), CmpOp::Ge, Num::Lit(4)),
            error: "CANNOT_USE_ATTACK",
        }),
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Erasure Ball: discard up to 2 Energy from your Benched Pokémon; 60 more damage for each.
        steps: &[
            Step::after_damage(Op::EnergyChoice(EnergyChoiceSpec {
                how: EnergyHow::Prompt { scope: PromptScope::Bench, min: Num::Lit(0), max: Num::Lit(2), kind: EnergyKind::Any, clamp: false },
                into: Some(0),
                ..EnergyChoiceSpec::DEFAULT
            })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), op: DamageOp::Add, per: 60 })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
