//! Basculin (WHT 24): Bite - 10. Bared Fangs - 50; if your opponent's Active Pokémon has no damage counters on it before this
//! attack does damage, this attack does nothing.
//!
//! Before the damage the attack's damage is set to 0 when the Defending Pokémon has no damage on it (the damage read before
//! this attack does any).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Basculin",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(0), when: Cond::Cmp(Num::DamageOn(OPP_ACTIVE), CmpOp::Eq, Num::Lit(0)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
