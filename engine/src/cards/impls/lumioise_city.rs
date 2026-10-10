//! Lumiose City (POR / M3, stadium): once during each player's turn, that
//! player may search their deck for a Basic Pokémon and put it onto their
//! Bench, then shuffle. If they do, their turn ends.
//!
//! Twinleaf: the turn ends (EndTurnEffect) after the shuffle even when no
//! Pokémon was chosen; the shuffle has no animation wait.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LumioiseCity",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[],
        steps: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })), Step::new(Op::EndTurn(EndTurnSpec { who: Who::Me }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
