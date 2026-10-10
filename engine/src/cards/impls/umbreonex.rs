//! Umbreon ex (PRE 60, Tera): Moon Mirage — 160; your opponent's Active Pokémon is now Confused. Onyx — discard all
//! Energy from this Pokémon, and take a Prize card.
//!
//! Moon Mirage's Confusion is a GainCondition by the attack. Onyx discards every Energy card on Umbreon ex, then one
//! TakePrizes event (the Prize prompt, or the last Prize card taken at once). Tera: `TERA_RULE`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Umbreonex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused]))] },
        AttackSpec {
            index: 1,
            // Onyx: discard all Energy from this Pokémon and take a Prize card.
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::TakePrize(TakePrizeSpec { who: Who::Me, count: Num::Lit(1) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
