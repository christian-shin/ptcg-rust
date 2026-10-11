//! Team Rocket's Chingling (DRI): Chiming Commotion — discard a random card from your opponent's hand (a Discard of the
//! attack's effect, after the damage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsChingling",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
