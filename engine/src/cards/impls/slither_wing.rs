//! Slither Wing (SFA): Iron Buster — 20+; 120 more if your opponent has a
//! Future Pokémon in play. Smashing Wings — 130, discard 2 Energy from this
//! Pokémon.
//!
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2): ChooseEnergyPrompt over the
//! Active's CheckProvidedEnergy map for [C][C] (no cancel), then a
//! DiscardCardsEffect aimed at the attacker's Active.
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
