//! Jolteon ex (PRE, Tera): Flashing Spear — 60+; you may discard up to 2 Basic Energy from your Benched Pokémon, 90 more
//! damage for each card discarded. Dravite — 280; during your next turn this Pokémon can't attack.
//!
//! Flashing Spear: no prompt without Basic Energy on the Bench; the discarded count is the bonus (before the damage).
//! Dravite arms `Lasting::CannotAttackNextTurn` (an ApplyEffect event on this Pokémon). The Tera rule is `TERA_RULE`:
//! a `Prevent` over `Kind(Damage)` on this Pokémon while it is on the Bench.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jolteonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::FromBench { max: 2, pred: Pred::BasicEnergy }, ..DiscardEnergySpec::DEFAULT })),
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Last, &Num::Lit(90)), when: Cond::True })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
