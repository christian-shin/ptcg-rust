//! Team Rocket's Mewtwo ex (DRI): Power Saver — can't attack unless you have
//! 4 or more Team Rocket's Pokémon in play. Erasure Ball — 160+; discard up
//! to 2 Energy from your Benched Pokémon, 60 more damage for each.
//!
//! Power Saver blocks using the attack while this Pokémon's Ability works. Erasure Ball offers any Energy (the text says
//! "Energy"), is skipped without a Benched Pokémon, and, id2423: the Energy is chosen first, the damage is done,
//! then the Energy is discarded.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMewtwoex",
    // Power Saver: this Pokémon can't attack unless you have 4 or more Team Rocket's Pokémon in play.
    // A lock over its own UseAttack while the condition doesn't hold (events batch 7).
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockUse(BlockUseSpec {
            binds: Binds::Owner,
            lock: LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::This(Role::Card)]), "CANNOT_USE_ATTACK"),
            while_: &[LockWhile::Unless(&Cond::Cmp(Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(tag::TEAM_ROCKET)), CmpOp::Ge, Num::Lit(4)))],
            ability: true,
        }),
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Erasure Ball: discard up to 2 Energy from your Benched Pokémon; 60 more damage for each.
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Scoped { scope: PromptScope::Bench, min: Num::Lit(0), max: Num::Lit(2), kind: EnergyKind::Any, clamp: false }, into: Some(0), ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), op: DamageOp::Add, per: 60 })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
