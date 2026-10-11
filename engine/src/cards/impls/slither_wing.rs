//! Slither Wing (SFA 26): Iron Smasher - 20+; if your opponent has any Future Pokémon in play, 120 more damage. Smashing
//! Wing - 130; discard 2 Energy from this Pokémon.
//!
//! Smashing Wing discards 2 Energy cards after the damage: a LeavePlay per card (D1), the player chooses which, cause: this
//! attack.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "SlitherWing",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(120, Cond::InPlay(Who::Opp, PlayScope::All, Pred::Tag(tag::FUTURE))))] },
        AttackSpec {
            index: 1,
            // Discard 2 Energy from this Pokémon (priced as [C][C]).
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cost { n: Num::Lit(2), ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
