//! Gimmighoul (SSP): Minor Errand-Running - search your deck for up to 2
//! Basic Energy cards, reveal them, put them into your hand, shuffle. Tackle - 50.
//!
//! Twinleaf: the ShuffleDeckPrompt is created right after the
//! ChooseCardsPrompt (before it is answered), with no trailing wait; the
//! ShowCards info prompt (only when any cards were chosen) is created in the
//! choose callback.
//!
//! Spec: the search shuffles right after its prompt opens, as Twinleaf does
//! (the replay's shuffle tape matches by deck size).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gimmighoul",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::BasicEnergy,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: true,
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
