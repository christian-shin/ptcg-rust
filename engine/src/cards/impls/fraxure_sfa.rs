//! Fraxure (SFA 45): Unnerve - whenever your opponent plays an Item or Supporter card from their hand, prevent all effects
//! of that card done to this Pokémon. Dragon Pulse - 80; discard the top card of your deck.
//!
//! Unnerve is a `Prevent` over every event an Item or Supporter of the opponent causes (`CauseKind::Trainer`) on this
//! Pokémon: the card is still played and its effect on this Pokémon doesn't happen (id284). A Supporter used as an attack's
//! effect isn't played from the hand and has the attack as its cause. Dragon Pulse discards the top card of your deck after
//! the damage (a Discard event caused by the attack; Haxorus SFA 46).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FraxureSFA",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::IsThisPokemon, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Trainer)])))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
