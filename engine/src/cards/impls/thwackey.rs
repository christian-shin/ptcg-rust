//! Thwackey (TWM): Boom Boom Groove — once during your turn, if your Active
//! Pokémon has the Festival Lead Ability, search your deck for a card and put
//! it into your hand, then shuffle. Beat — 50.
//!
//! Twinleaf: Festival Lead is looked up by name on the Active's printed
//! powers; with no Active Pokémon the ability silently does nothing. The
//! marker is cleared when this card is played and at its owner's end of
//! turn. The final ShuffleDeckPrompt has no trailing wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Thwackey",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BOOM_BOOM_DRUM_MARKER"),
        // If your Active Pokémon has the Festival Lead Ability.
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Slot(MY_ACTIVE, SlotPred::Top(Pred::HasAbilityNamed("Festival Lead")))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
