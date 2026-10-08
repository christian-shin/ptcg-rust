//! Haxorus (SFA): Bring Down the Axe — if the opponent's Active has any
//! Special Energy attached it is Knocked Out (Mist-blockable
//! KnockOutOpponentEffect). Dragon Pulse — 230; discard the top 3 cards of
//! your deck (deck -> scratch CardList -> discard, two MOVE_CARDS).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Haxorus@Haxorus SFA",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::KnockOut(KnockOutSpec { target: SlotExpr::Active(Who::Opp), mode: KnockOutMode::Opponent, when: Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Opp)), EnergyUnit::SpecialEnergyCards), CmpOp::Gt, Num::Lit(0)) })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Top(Num::Lit(3)), ..MoveSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
