//! Mega Meganium ex (MC / ASC 10): Giant Bouquet — 70+; 50 more for each
//! [G] Energy attached to this Pokémon.
//!
//! Twinleaf: `CheckProvidedEnergyEffect(player)` (source defaults to
//! `player.active`); counts GRASS or ANY `provides` entries and sets
//! `effect.damage = 70 + 50 * count`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaMeganiumex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Add(&Num::Lit(70), &Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(50))), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
