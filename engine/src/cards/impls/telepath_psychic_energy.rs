//! Telepathic Psychic Energy (POR 88, Special Energy): provides [P]. When you attach this card from your hand to a [P] Pokémon,
//! search your deck for up to 2 Basic [P] Pokémon and put them onto your Bench. Then, shuffle your deck.
//!
//! Only an Attach from the hand triggers it, and the [P] type is read on the Pokémon it is attached to. The search is for a kind
//! (up to 2, may find none; Bench space limits it) and the deck is shuffled after it whenever the deck isn't empty (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TelepathPsychicEnergy",
    passives: &[Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::PSYCHIC])] }) }],
    // When attached to a [P] Pokémon, search your deck for up to 2 Basic [P] Pokémon and put them onto your Bench, then shuffle.
    triggers: &[Trigger {
        origin: RuleSource::Energy,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Slot(SlotExpr::Picked, SlotPred::TypeIs(ct::PSYCHIC)), Cond::BenchSpace(Who::Me), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Basic, Pred::PokemonType(ct::PSYCHIC)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
                    cancel: true,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
