//! Hydrapple (DRI 18): Hydra Breath — discard 6 Basic [G] Energy from your
//! hand in order to Knock Out your opponent's Active Pokémon. Whip Smash — 140.
//!
//! Twinleaf: counts Basic energy cards named 'Grass Energy' in hand; with 6+
//! a non-cancellable ChooseCardsPrompt (exactly 6) from hand, then
//! MOVE_CARDS to the discard and a KnockOutOpponentEffect on the opponent's
//! Active.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hydrapple",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])), CmpOp::Ge, Num::Lit(6)), yes: &[Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]), bounds: Bounds { min: Num::Lit(6), max: Num::Lit(6) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })), Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Active(Who::Opp), when: Cond::True }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
