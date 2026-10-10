//! Hearthflame Mask Ogerpon ex (TWM 40 / PRE 17, Tera): Wrathful Hearth —
//! 20× damage for each damage counter on this Pokémon. Dynamic Blaze — 140+; if
//! your opponent's Active Pokémon is an Evolution Pokémon, this attack does 140
//! more damage, and discard all Energy from this Pokémon. Tera: as long as this
//! Pokémon is on your Bench, prevent all damage done to it by attacks.
//!
//! Rule: Wrathful Hearth does 20 damage per damage counter on this Pokémon. Both
//! parts of Dynamic Blaze depend on the condition (the discard is a Discard event
//! of the Energy this Pokémon provides, cause: this attack, after the damage). The
//! Tera rule is `TERA_RULE`: a `Prevent` over `Kind(Damage)` while Benched.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HearthflameMaskOgerponex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::DamageOn(SlotExpr::This), &Num::Lit(2)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::before_damage(more_damage_if(140, Cond::Not(&Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::Basic)))),
            Step::after_damage(Op::If(IfSpec { cond: Cond::Not(&Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::Basic)), yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))], no: &[] })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
