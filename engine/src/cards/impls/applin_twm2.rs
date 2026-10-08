//! Applin (TWM 126): Find a Friend — search your deck for a Pokémon, reveal
//! it, put it into your hand, then shuffle. Rolling Tackle — 30.
//!
//! Twinleaf: the ShowCardsPrompt to the opponent is created even when no
//! card was chosen; the ShuffleDeckPrompt follows with no trailing wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Applin@Applin TWM2",
    // Find a Friend: search your deck for a Pokémon, reveal it, put it into your hand, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Pokemon, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    msg: "",
                    cancel: false,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
