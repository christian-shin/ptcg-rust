//! Drilbur (TEF 85): Dig Dig Dig: when you play this Pokémon from your hand onto your Bench during your turn, you may
//! search your deck for up to 3 Basic [F] Energy cards and discard them. Then, shuffle your deck. Sand Spray: 20.
//!
//! A trigger over this card's EnterPlay from the hand to the Bench by the rules (not an effect). It asks first (a "may", only
//! with a card in the deck); the chosen Energy go from the deck to the discard pile (a Discard event, this Ability its cause),
//! then the deck is shuffled, even when none was taken (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "DrilburTEF",
    // Dig Dig Dig: when you play this Pokémon from your hand onto your Bench, you may search your
    // deck for up to 3 Basic [F] Energy cards and discard them, then shuffle.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec {
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Discard { reveal: false },
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
