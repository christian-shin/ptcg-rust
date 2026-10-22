//! Durant (WHT 70): Bite Together - 20+; if Durant is on your Bench, this attack does 20 more damage. Vise Grip - 50.
//!
//! "Durant" is the card's name for a Pokémon named Durant: one on the Bench adds 20, however many there are (id2188).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Durant",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(20, Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Durant"))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
