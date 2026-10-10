//! Noctowl (SCR): Jewel Seeker — when you play this Pokémon from your hand to
//! evolve 1 of your Pokémon, if you have any Tera Pokémon in play, you may
//! search your deck for up to 2 Trainer cards, reveal them, and put them into
//! your hand. Then, shuffle your deck. Speed Wing — 60.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Noctowl",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            // It needs a card in the deck and a Tera Pokémon in play.
            cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA))]),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Trainer, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        msg: "",
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
