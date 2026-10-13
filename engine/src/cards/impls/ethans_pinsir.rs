//! Ethan's Pinsir (DRI 1): Vise Grip — 20. Rallying Horn — 70+; if any of your Ethan's Pokémon were Knocked Out by damage from an
//! attack during your opponent's last turn, 100 more damage.
//!
//! `Cond::KnockedOutLastTurn` with `by_attack_damage` and the Ethan's tag, read from the opponent's last turn's Knock Out
//! record (a Knock Out by an attack's damage of a Pokémon with the tag).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansPinsir",
    // One-Point Return: 100 more damage if any of your Ethan's Pokémon were Knocked Out by damage
    // from an attack during your opponent's last turn.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(more_damage_if(
            100,
            Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: Some(crate::types::tag::ETHANS) },
        ))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
