//! Jynx (MEG 57): Psychic - 30+; this attack does 30 more damage for each Energy attached to your opponent's Active Pokémon.
//!
//! Adds 30 per Energy unit on the opponent's Active Pokémon before the damage (Rabsca TEF 24).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Jynx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
