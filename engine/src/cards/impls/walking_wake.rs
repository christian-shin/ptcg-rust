//! Walking Wake (TWM): Aurora Gain — 20; heal 20 damage from this Pokémon
//! (a HealEffect on the Active). Undulating Slice — put up to 9 damage
//! counters on this Pokémon; 20 damage for each counter placed.
//!
//! Twinleaf: Undulating Slice is a non-cancellable PutDamagePrompt (90 in
//! multiples of 10, partial placement allowed, Active slot only) with a
//! per-Pokémon cap of CheckHp + 90 for every Pokémon in play; each entry is a
//! PutCountersEffect on the chosen target and `effect.damage = placed * 2`
//! (the last entry wins).
//!
//! Fixed (phase 4b, W4): the printed damage is 20 ("20×", as on the card), so
//! the resume sets `effect.damage = 0` before the entries (placing no counters
//! does 0 damage, not the printed 20).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WalkingWake",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(20), via: HealVia::Effect, clear_conditions: false })),
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
