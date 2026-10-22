//! Heatran (PBL 7): Singe - your opponent's Active Pokémon is now Burned. Lava Wall - 120; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Burned Pokémon.
//!
//! Singe: a GainCondition(Burned). Lava Wall arms the lasting prevention `DamageSource::Burned` on this Pokémon: attacks
//! from a Pokémon that is Burned when it damages this Pokémon (a checked read) do no damage; others do.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HeatranPBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Burned) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
