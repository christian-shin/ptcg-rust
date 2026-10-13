//! Seaking (PRE): Festival Lead — if Festival Grounds is in play, this Pokémon may use an attack it has twice. If the first
//! attack Knocks Out your opponent's Active Pokémon, you may attack again after your opponent chooses a new Active Pokémon.
//! Rapid Draw — 60; draw 2 cards.
//!
//! Festival Lead is a declaration the core reads when the second attack would be offered (`Modifier::AttackTwice`). Rapid Draw is one Draw event of the attack's effect, after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Seaking",
    // Festival Lead: if Festival Grounds is in play, this Pokémon may use an attack it has twice (read by the core when
    // the second attack would be offered: `attack::attack_twice`).
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackTwice(AttackTwiceSpec { guard: Cond::StadiumInPlay(Pred::Name("Festival Grounds")) }) }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
