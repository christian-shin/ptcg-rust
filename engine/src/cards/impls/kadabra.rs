//! Kadabra (MEG): Psychic Draw — once during your turn, when you play this Pokémon from your hand to evolve 1 of your
//! Pokémon, you may use this Ability. Draw 2 cards. Super Psy Bolt — 30.
//!
//! A trigger on the Evolve event of this card with Source Hand (evolving from the deck, by Grand Tree or an attack, doesn't
//! fire it); the Draw is one Draw event of the Ability's effect.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Kadabra",
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Not(&Cond::AbilityBlocked)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
