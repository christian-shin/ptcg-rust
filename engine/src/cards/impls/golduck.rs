//! Golduck (ASC 40 / MEP 8): Damp (see Psyduck). Hydro Pump — 60+; 20 more
//! damage for each [W] provided by the Energy attached to this Pokémon
//! (CheckProvidedEnergy on the attacking player's Active).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Golduck@ASC|MEP",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::Provided(ct::WATER)), &Num::Lit(20)), when: Cond::True })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AbilityLock(DAMP) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
