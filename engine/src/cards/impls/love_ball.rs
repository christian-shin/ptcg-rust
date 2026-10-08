//! Love Ball (TWM): search your deck for a Pokémon with the same name as 1
//! of your opponent's Pokémon in play, reveal it, and put it into your hand.
//! Then, shuffle your deck.
//!
//! Twinleaf quirks kept: no preventDefault; the choice is min 1 / max 1 and
//! can be cancelled; MOVE_CARDS runs even with nothing chosen, then the
//! reveal, then the shuffle.
//!
//! Fixed (phase 4b, rulings 336/1285): unplayable when all 4 copies of every name the
//! opponent has in play are in known zones.
//!
//! Fixed (phase 4b, R2): the allowed names were only the name of
//! `opponent.active.cards[0]` (the bottom card of the stack, so the Basic of
//! an evolved Pokémon) because `opponent.bench.filter(card instanceof
//! PokemonCard)` is always empty (bench entries are card lists); the allowed
//! names are now the top Pokémon's name of every Pokémon the opponent has in
//! play (forEachPokemon).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LoveBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Not(&Cond::AllNamesKnown { names_of: Who::Opp })],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, same_name_as: Some(Who::Opp), ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: true,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
