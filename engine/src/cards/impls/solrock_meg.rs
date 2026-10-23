//! Solrock (MEG, the Solrock of ASC 106): Cosmic Beam — 70. If you don't have Lunatone on your Bench, this attack does nothing.
//! This attack's damage isn't affected by Weakness or Resistance.
//!
//! The two attack flags (no Weakness, no Resistance) are set before the damage. Lunatone is read on the Bench only, as printed (the
//! Active Pokémon using the attack is Solrock itself): without one the attack's damage is 0 and nothing else happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SolrockASC",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            // "If you don't have Lunatone on your Bench, this attack does nothing."
            Step::before_damage(Op::If(IfSpec { cond: Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Lunatone")), yes: &[], no: &[Step::new(attack_does_nothing(Cond::True))] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
