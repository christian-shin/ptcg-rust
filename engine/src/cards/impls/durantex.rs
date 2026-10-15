//! Durant ex (SSP 4): Sudden Shearing - when you play this Pokémon from your hand onto your Bench during your turn,
//! you may discard the top card of your opponent's deck. Vengeful Crush - 120+; 30 more damage for each Prize card your
//! opponent has taken.
//!
//! Sudden Shearing is a trigger on this card's EnterPlay (source: the hand, by the player's own action, onto the Bench);
//! it is asked unless the opponent's deck is empty. The card is a Discard event from the opponent's deck (cause: this
//! Ability).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DurantEx",
    // Sudden Shearing: when you play this Pokémon from your hand onto your Bench, you may discard
    // the top card of your opponent's deck.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT }))],
            no: &[],
        }))],
    }],
    // Vengeful Crush: 30 more damage for each Prize card your opponent has taken.
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(30)), when: Cond::True }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
