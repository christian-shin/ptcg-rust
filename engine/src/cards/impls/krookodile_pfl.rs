//! Krookodile (PFL 66): Vengeful Fang - 60+; if any of your Pokémon were Knocked Out by damage from an attack during your opponent's
//! last turn, this attack does 160 more damage. Hammer In - 160. This is a different card from Krookodile BLK 59.
//!
//! The condition is the record of the opponent's last turn (Pokémon of the player Knocked Out by damage from an attack, one of any
//! kind: id2245, a Knock Out whose Prize was reduced still counts); read before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KrookodilePFL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(160, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: None }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
