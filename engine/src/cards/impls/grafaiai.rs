//! Grafaiai (MEG 92): Miraculous Paint - 90; flip a coin. If heads, choose a Special Condition. Your opponent's Active Pokémon is now
//! affected by that Special Condition.
//!
//! After the damage the coin is flipped; on heads the player chooses 1 of the five Special Conditions (a Select prompt, all offered)
//! and the Special Condition event is the attack's (Cradily SCR 6's Selective Slime with five branches).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Grafaiai",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Choose(ChooseSpec {
                chooser: Who::Me,
                options: &[
                    ChoiceBranch { label: "ASLEEP", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Asleep]))] },
                    ChoiceBranch { label: "BURNED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Burned]))] },
                    ChoiceBranch { label: "CONFUSED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Confused]))] },
                    ChoiceBranch { label: "PARALYZED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))] },
                    ChoiceBranch { label: "POISONED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Poisoned]))] },
                ],
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
