//! Glaceon (TWM 54): Permeating Chill — 30; at the end of your opponent's next turn, put 9 damage counters on the Defending
//! Pokémon. Icicle Missile — 70.
//!
//! The counters are an effect on the Defending Pokémon (the Slow Crunch / Intense Kiss family: it ends when that Pokémon moves
//! to the Bench or evolves, id1981, id2404), placed as one of the end-of-turn effects of the opponent's next turn (APR E-21);
//! counters aren't damage (APR C-07): no Weakness, Resistance or damage modifiers. A Knock Out by them is the state check's,
//! after every end-of-turn effect.
use crate::spec::prelude::*;

static PERMEATING_CHILL: EndOfTurnSpec = EndOfTurnSpec { steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(9) }))] };

pub static SPEC: CardSpec = CardSpec {
    class: "GlaceonTWM",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::AtEndOfOpponentsNextTurn(&PERMEATING_CHILL) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
