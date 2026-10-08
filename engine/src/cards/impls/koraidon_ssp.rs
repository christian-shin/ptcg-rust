//! Koraidon (SSP): Unrelenting Onslaught — 30+; 150 more if 1 of your other
//! Ancient Pokémon used an attack during your last turn. Hammer In — 110.
//!
//! BOOST_IF_OTHER_ANCIENT_ATTACKED_LAST_TURN: `ancientPokemonAttackedLastTurn`
//! and `playerLastAttack[player].sourceCard` is another (Ancient) card.
//! Phase 4b: the flag is set at the end of a turn only when the last attack was
//! used during that turn (`playerLastAttack.turn`); a turn that ended without
//! an attack used to keep the previous Ancient attack's flag alive, and a
//! failed attack attempt while Confused (tails) voids the stamp (ruling n=1621).
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
