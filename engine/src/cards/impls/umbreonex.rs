//! Umbreon ex (PRE, Tera): Moon Mirage — 160; your opponent's Active
//! Pokémon is now Confused. Onyx — discard all Energy from this Pokémon and
//! take a Prize card. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Onyx builds the energy map of `player.active`, reduces a
//! DiscardCardsEffect (target = the attacker's Active), then TAKE_X_PRIZES
//! (1): with 1 Prize left it is taken automatically, otherwise a
//! non-cancellable ChoosePrizePrompt (not secret).
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
