//! Enriching Energy (Rich Energy SSP, ACE SPEC): provides [C]; when you
//! attach it from your hand to one of your Pokémon, draw 4 cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RichEnergy",
    triggers: &[Trigger {
        origin: RuleSource::Energy,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(4)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
