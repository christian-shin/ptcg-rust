//! Haxorus (SFA): Bring Down the Axe — if the opponent's Active has any
//! Special Energy attached it is Knocked Out. Dragon Pulse — 230; discard the
//! top 3 cards of your deck.
//!
//! Bring Down the Axe is `Op::KnockOut` on the opponent's Active Pokémon: a KnockOut by an effect, asked the preventions
//! when the attack's effect runs (Mist Energy stops it, id2427) and otherwise waiting for the state check (D1).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Haxorus@SFA",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::KnockOut(KnockOutSpec { target: SlotExpr::Active(Who::Opp), when: Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Opp)), EnergyUnit::SpecialEnergyCards), CmpOp::Gt, Num::Lit(0)) })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(3)), ..DiscardSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
