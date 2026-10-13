//! Hisuian Arcanine (TWM): Proud Fangs — 30+; 90 more if your Benched
//! Pokémon have any damage counters. Searing Flame — 90, the opponent's
//! Active Pokémon is now Burned.
//!
//! Proud Fangs adds 90 to the damage when any of your Benched Pokémon has damage counters (`Cond::AnySlot` over the Bench,
//! `SlotPred::Damaged`), before the damage. Searing Flame's Burn is an effect of the attack: a GainCondition event with the attack
//! as its cause (Mist Energy and the like prevent it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HisuianArcanine",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(90), when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Damaged) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
