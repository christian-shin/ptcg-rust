//! Koraidon (SSP): Unrelenting Onslaught - 30+; 150 more if 1 of your other Ancient Pokémon used an attack during your last turn.
//! Hammer In - 110.
//!
//! `Cond::OtherAncientAttackedLastTurn`: the attack the player used during their previous turn was used by an Ancient Pokémon
//! other than this one (a record of the player's last attack and its turn; the flag it reads is set when the turn ends). A turn
//! that ended without an attack leaves no bonus, and a failed attack attempt while Confused (tails) is not an attack used
//! (id2087, JP FAQ Koraidon SSP / TEF).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Koraidon@SSP",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(150, Cond::OtherAncientAttackedLastTurn)),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
