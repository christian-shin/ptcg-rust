//! Solrock (MEG, as Solrock ASC): Cosmo Beam — 70. If you don't have Lunatone
//! on your Bench, this attack does nothing. This attack's damage isn't
//! affected by Weakness or Resistance.
//!
//! Twinleaf (mega-evolution file): sets ignoreWeakness/ignoreResistance, then
//! looks for a Lunatone anywhere among your Pokémon in play (Active included,
//! not only the Bench); none → damage 0.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Solrock@ASC",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            // Lunatone anywhere in play (the Active included), not only on the Bench.
            Step::before_damage(Op::If(IfSpec { cond: Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("Lunatone")), yes: &[], no: &[Step::new(damage_is(Num::Lit(0)))] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
