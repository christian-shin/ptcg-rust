//! Arven's Toedscruel (DRI 110): Pull - switch in 1 of your opponent's Benched Pokémon to the Active Spot. Reckless Charge - 120; this Pokémon also does 30 damage to itself.
//!
//! Pull is a SwitchIn done to the Benched Pokémon the attacker chooses (Clefairy POR 30's Follow Me); Reckless Charge's self damage follows Arrokuda DRI 62.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ArvensToedscruel",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, required: false })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
