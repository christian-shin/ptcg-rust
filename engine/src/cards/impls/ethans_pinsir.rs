//! Ethan's Pinsir (DRI 1): Vice Grip — 20. One-Point Return — 70+; 100 more
//! if any of your Ethan's Pokémon were Knocked Out by damage from an attack
//! during your opponent's last turn.
//!
//! Twinleaf: WAS_POKEMON_KNOCKED_OUT_DURING_OPPONENTS_LAST_TURN with
//! `{ byAttackDamage: true, tags: [ETHANS] }`: the by-attack flag, then any
//! KO entry (all KOs of that turn, not only attack KOs) carrying the tag.
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
