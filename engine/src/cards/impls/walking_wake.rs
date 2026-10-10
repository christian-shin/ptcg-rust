//! Walking Wake (TWM): Aurora Gain — 20; heal 20 damage from this Pokémon. Undulating Slice — put up to 9 damage counters
//! on this Pokémon; 20 damage for each counter placed.
//!
//! Undulating Slice is a PutDamage prompt (up to 90 in multiples of 10, partial placement allowed, this Pokémon only,
//! cap = its HP + 90, id1984): one PlaceCounters event caused by the attack (counters are not damage, APR C-07), then
//! the attack's damage is 20 per counter placed (0 placing none: the printed 20 is "20x"). Counters beyond its HP are
//! allowed; the state check Knocks it Out at the end of the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WalkingWake",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(20), clear_conditions: false })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::SpreadCounters(SpreadCountersSpec { chooser: Who::Me, total_hp: 90, cap_bonus_hp: 90, damage_per_hp: 2 })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
