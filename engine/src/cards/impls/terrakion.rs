//! Terrakion (SV11W 54): Retaliate — 50+; 80 more if any of your Pokémon
//! were Knocked Out by damage from an attack during your opponent's last
//! turn. Land Crush — 100.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Terrakion",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(more_damage_if(80, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: None }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
