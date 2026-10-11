//! Levincia (DRI 244, Stadium): once during each player's turn, that player may put up to 2 Basic [L] Energy cards from
//! their discard pile into their hand.
//!
//! A Stadium use (UseStadium), once per turn for each player. The Stadium's "up to 2" is 1 or 2 (id2301). The chosen cards
//! are put into the hand by one PutIntoHand event from the discard pile (cause: the Stadium). It can be used only with a
//! Basic [L] Energy card in the discard pile (an effect that can't change the game can't be used, id925).
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
            }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
