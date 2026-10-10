//! Dragapult ex (TWM): Phantom Dive - 200, and put 6 damage counters on your
//! opponent's Benched Pokémon in any way you like. Tera: no damage from
//! attacks while on the Bench.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dragapultex",
    // Phantom Dive: put 6 damage counters on your opponent's Benched Pokémon in any way you like.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::SpreadDamage(SpreadDamageSpec {
            chooser: Who::Me,
            side: Who::Opp,
            slots: SpreadSlots::Bench,
            total_hp: 60,
            unit_hp: 10,
            cap_bonus_hp: None,
            apply: SpreadApply::Counters,
        }))],
    }],
    // Tera: no attack damage while Benched.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
