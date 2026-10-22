//! Bronzong (PFL 72): Triple Draw — draw 3 cards. Tool Drop — 40×; this attack does 40 damage for each Pokémon Tool attached to
//! all Pokémon.
//!
//! Triple Draw is one Draw event of 3 cards (cause: the attack). Tool Drop sets the damage to 40 times the Pokémon Tools attached
//! to every Pokémon in play, both players' (`Num::ToolsInPlay(None)`), before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BronzongPFL",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))] },
        AttackSpec { index: 1, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::ToolsInPlay(None), &Num::Lit(40))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
