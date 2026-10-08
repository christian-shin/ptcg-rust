//! Levincia (JTG / DRI, stadium): once during each player's turn, that player
//! may put up to 2 Basic [L] Energy cards from their discard pile into their
//! hand.
//!
//! Twinleaf: throws CANNOT_USE_POWER unless the discard pile holds a basic
//! Energy providing [L]; the ChooseCardsPrompt (name "Lightning Energy",
//! min 1, max 2, no cancel: up to 2 from a public zone, rulings 1778/1853) is followed by a MOVE_CARDS to the hand (an
//! empty selection still reduces a MoveCardsEffect).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Levincia",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::LIGHTNING)]))],
        steps: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
                shuffle_first: false,
            }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
