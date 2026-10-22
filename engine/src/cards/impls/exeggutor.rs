//! Exeggutor (MEG 5): Guard Press - 30; during your opponent's next turn, this Pokémon takes 30 less damage from attacks (after applying Weakness and Resistance). Stomping Wood - 60+; this attack does 30 more damage for each [G] Energy attached to this Pokémon.
//!
//! `Lasting::TakesLessDamage(30)` (Clamperl DRI 54); 30 more per [G] Energy this Pokémon provides (Golduck ASC 40).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Exeggutor",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(30)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
