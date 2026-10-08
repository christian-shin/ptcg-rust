//! Dipplin (TWM 18): Festival Lead — if Festival Grounds is in play, this
//! Pokémon may use an attack twice. Do the Wave — 20 damage for each of your
//! Benched Pokémon.
//!
//! Twinleaf: Festival Lead is the attack's `barrage` flag, written on the
//! card object whenever Do the Wave's AttackEffect is reduced. Fixed (R1-7):
//! while the Ability is blocked the flag is written `false` (it used to be
//! left as an earlier use set it, so a blocked Dipplin could still attack
//! twice). The damage is only recomputed when the opponent's Active holds a
//! Pokémon.
use crate::spec::prelude::*;


pub static SPEC: CardSpec = CardSpec {
    class: "Dipplin@Dipplin TWM1|Dipplin PRE",
    // Festival Lead: if Festival Grounds is in play, this Pokémon may use an attack twice. Do the
    // Wave: 20 damage for each of your Benched Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::BenchCount(Who::Me), &Num::Lit(20)))),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::FestivalLead, value: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
