//! Love Ball (TWM 156): search your deck for a Pokémon with the same name as 1 of your opponent's Pokémon in play, reveal
//! it, and put it into your hand. Then, shuffle your deck.
//!
//! Usable only when a Pokémon with a name an opponent's Pokémon in play has can still be in the deck: when all 4 copies of
//! every such name are in known zones the effect can't be done (id361: a search for a card known not to be there isn't
//! made; usability by the rules, id925). The chosen Pokémon is revealed and goes into the hand (PutIntoHand), then the deck
//! is shuffled (APR E-19).
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
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
