//! Seaking (PRE): Festival Lead — if Festival Grounds is in play, this Pokémon may use an attack it has twice. If the first
//! attack Knocks Out your opponent's Active Pokémon, you may attack again after your opponent chooses a new Active Pokémon.
//! Rapid Draw — 60; draw 2 cards.
//!
//! Festival Lead is the attack's repeat flag, set before the damage while Festival Grounds is in play and the Ability works
//! (cleared otherwise: a blocked Seaking attacks once). Rapid Draw is one Draw event of the attack's effect, after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Seaking",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            // Festival Lead: with Festival Grounds in play (and the Ability working) this attack may be used twice.
            Step::before_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::StadiumInPlay(Pred::Name("Festival Grounds")), Cond::Not(&Cond::AbilityBlocked)]),
                yes: &[Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::Barrage, value: true }))],
                no: &[Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::Barrage, value: false }))],
            })),
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
