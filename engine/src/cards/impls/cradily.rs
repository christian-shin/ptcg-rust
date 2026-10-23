//! Cradily (SCR 6): Selective Slime - once during your turn, you may flip a coin. If heads, choose Burned, Confused, or Poisoned. Your
//! opponent's Active Pokémon is now affected by that Special Condition. Miasma Wind - 100x; this attack does 100 damage for each
//! Special Condition affecting your opponent's Active Pokémon.
//!
//! Selective Slime can't be used when the Defending Pokémon is already Burned, Confused and Poisoned (id2014: nothing could change,
//! `Cond::WouldChangeConditions`); the coin is flipped (a CoinFlip by the Ability), on heads the player chooses one of the three
//! (a Select prompt) and the Special Condition event is the Ability's (a Pokémon that can't be Confused keeps it out). Miasma Wind
//! counts the Special Conditions on the Defending Pokémon before the damage (`Num::Conditions`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cradily",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Poisoned])],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Choose(ChooseSpec {
                chooser: Who::Me,
                options: &[
                    ChoiceBranch { label: "BURNED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Burned]))] },
                    ChoiceBranch { label: "CONFUSED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Confused]))] },
                    ChoiceBranch { label: "POISONED", avail: Cond::True, body: &[Step::new(inflict(&[SpecialCondition::Poisoned]))] },
                ],
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::Conditions(OPP_ACTIVE), &Num::Lit(100))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
