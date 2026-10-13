//! Dipplin (TWM 18): Festival Lead — if Festival Grounds is in play, this Pokémon may use an attack twice. Do the Wave — 20
//! damage for each of your Benched Pokémon.
//!
//! Festival Lead is a declaration (`Modifier::AttackTwice`) the core reads when the second attack would be offered, from
//! this Pokémon, for an attack printed on it (batch 8 C12c; JP FAQ Dipplin / Flutter Mane x3, Mew ex: never through a
//! copy). Do the Wave sets the damage to 20 per Benched Pokémon before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dipplin@TWM|PRE",
    // Festival Lead: if Festival Grounds is in play, this Pokémon may use an attack it has twice (read by the core when
    // the second attack would be offered: `attack::attack_twice`).
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackTwice(AttackTwiceSpec { guard: Cond::StadiumInPlay(Pred::Name("Festival Grounds")) }) }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::BenchCount(Who::Me), &Num::Lit(20)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
