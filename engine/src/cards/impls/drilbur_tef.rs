//! Drilbur (TEF): Dig Dig Dig — when you play this Pokémon from your hand
//! onto your Bench during your turn, you may search your deck for up to 3
//! Basic [F] Energy cards and discard them, then shuffle. Sand Spray — 20.
//!
//! Twinleaf: on its PlayPokemonEffect (returns early on an empty deck or a
//! blocked ability, checked while the card is still in hand) a Confirm
//! prompt; yes → ChooseCardsPrompt (min 0, max 3, no cancel, basic energy
//! named "Fighting Energy"); MOVE_CARDS deck→discard when any was chosen, then
//! a bare ShuffleDeckPrompt (phase 4b: choosing nothing used to skip the
//! shuffle).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Drilbur@TEF",
    // Dig Dig Dig: when you play this Pokémon from your hand onto your Bench, you may search your
    // deck for up to 3 Basic [F] Energy cards and discard them, then shuffle.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(DECK, Pred::Any),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec {
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Discard { reveal: false },
                    msg: "CHOOSE_CARD_TO_HAND",
                    cancel: false,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
