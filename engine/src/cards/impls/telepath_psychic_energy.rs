//! Telepathic Psychic Energy (Telepath [P] Energy M3 / POR): provides [P].
//! When attached from your hand to a [P] Pokémon, search your deck for up to
//! 2 Basic [P] Pokémon and put them onto your Bench, then shuffle.
//!
//! Only an attach from the hand triggers it; the type check runs before the
//! card is attached. Phase 4b: the search
//! always happens when the deck is not empty (it used to be skipped, without a
//! shuffle, when the deck held no Basic [P] Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TelepathPsychicEnergy",
    passives: &[Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::PSYCHIC])], probe: false }) }],
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
                    msg: "",
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
