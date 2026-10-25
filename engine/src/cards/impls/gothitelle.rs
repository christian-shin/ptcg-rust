//! Gothitelle (WHT 43): Distorted Future - once during your turn, if this Pokémon is in the Active Spot, you may have your opponent
//! shuffle their hand into their deck and draw 3 cards. Synchro Shot - 90+; if you have the same number of cards in your hand as your
//! opponent, this attack does 90 more damage.
//!
//! Distorted Future: once per turn for each copy, printed for the Active Spot only (`Cond::Printed`); the opponent's whole hand goes
//! into their deck, the deck is shuffled and they draw 3 (Team Rocket's Archer DRI 148's form for the opponent). Synchro Shot compares
//! the two hands' sizes before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gothitelle",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Printed(&Cond::IsActive(SlotExpr::This))],
        steps: &[Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(3) }))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Eq, Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
