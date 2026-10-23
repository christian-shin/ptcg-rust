//! Mismagius ex (PFL 36): Swirling Prose - as long as this Pokémon is in the Active Spot, whenever your opponent's Active
//! Pokémon moves to the Bench during their turn, their new Active Pokémon is now Confused. Hexa-Magic - 150; you may draw
//! cards until you have 6 cards in your hand.
//!
//! Swirling Prose is a trigger over the ChangeActive of the opponent's Active Pokémon to the Bench (a retreat or any
//! switch) during the opponent's turn; it checks that this Pokémon is in the Active Spot when the change is done (id2339: a
//! switch that happens before this Pokémon is Active doesn't use it) and Confuses the Active Pokémon the opponent has then.
//! A Knock Out's replacement isn't a move to the Bench (id2247). Hexa-Magic's draw follows Cynthia's Garchomp ex ASC 111.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MismagiusEx",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::ChangeActive), EventPred::Owner(Who::Opp), EventPred::Turn(TurnOf::Opp), EventPred::From(SlotPred::Any)])),
        steps: &[Step::new(Op::If(IfSpec { cond: Cond::IsActive(SlotExpr::This), yes: &[Step::new(inflict(&[SpecialCondition::Confused]))], no: &[] }))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::All(&[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(6)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]),
            yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
