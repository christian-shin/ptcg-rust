//! Lumiose City (POR, Stadium): once during each player's turn, that player may search their deck for a Basic Pokémon and put
//! it onto their Bench. Then, that player shuffles their deck. If a player searches their deck in this way, their turn ends.
//!
//! A Stadium use (UseStadium). The turn ends after the shuffle even when no Pokémon was chosen (JP FAQ Lumiose City).
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
