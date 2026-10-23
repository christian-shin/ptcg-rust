//! Team Rocket's Grimer (DRI 123): Corrosive Sludge — at the end of your opponent's next turn, discard the Defending Pokémon
//! and all attached cards.
//!
//! APR E-21's own example of an end-of-turn effect: one of the end-of-turn effects of the opponent's next turn, ordered by that
//! player when there are several. The same text as Jynx PBL 32's Intense Kiss, the same effect (id2404: it is on the Defending
//! Pokémon and ends when that Pokémon moves to the Bench); a LeavePlay to its owner's discard pile, no Prize cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsGrimer",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::AtEndOfOpponentsNextTurn(&DISCARD_THE_DEFENDING_POKEMON) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
