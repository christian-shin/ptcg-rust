//! Octillery (CRI 17): Jet of Ink - 30; during your opponent's next turn, if the Defending Pokémon tries to use an attack,
//! your opponent flips 2 coins. If either of them is tails, that attack doesn't happen. Tantrum - 120; this Pokémon is now
//! Confused.
//!
//! Jet of Ink leaves the coin-gated lock of Sandslash MEG 69 / Hippopotas POR 39 with 2 coins (`attack_coin_gate_of(2)`:
//! one rule, the attacking player flips N coins and any tails stops the attack): the flips come before the Confusion flip
//! (id2317) and before the attack's costs (id497); one tails ends the attack and stops all its effects, two heads it goes
//! on (id606); the effect is on the Defending Pokémon (id1774). Tantrum confuses this Pokémon after the damage.
use crate::spec::prelude::*;

static JET_OF_INK: LockDecl = attack_coin_gate_of(2);

pub static SPEC: CardSpec = CardSpec {
    class: "OctilleryCRI",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&JET_OF_INK)) }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Confused]), when: Cond::True }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
