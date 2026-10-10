//! Kirlia (M1S / ASC): Call Sign - search your deck for up to 3 Pokémon and
//! put them into your hand, then shuffle. Psyshot - 30.
//!
//! Twinleaf: a ChooseCardsPrompt over the deck (Pokémon, min 0, max 3, no
//! cancel; nothing is revealed), MOVE_CARDS to the hand, then a bare
//! ShuffleDeckPrompt (no trailing wait).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kirlia@MEG|ASC",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
            })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
