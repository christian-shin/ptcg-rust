//! Forest of Vitality (POR, class LushForest, stadium): each player's [G]
//! Pokémon can evolve into [G] Pokémon during the turn they play those
//! Pokémon, except during their first turn.
//!
//! Events batch 2: a `Permit` on the Evolve event, a [G] Pokémon in play (its
//! type as the game checks it) into a [G] card, lifting the "came into play
//! this turn" limit (also for a Pokémon evolved this turn: id2229) on every
//! evolution path subject to it (the text doesn't say "from your hand";
//! id2327, docs/rulings/RULES.md "Evolution timing"). The first-turn limit
//! stays. It used to rewrite the played turn of every [G] Pokémon when a [G]
//! card was played, which let a [G] Pokémon evolve into a non-[G] one and let
//! Rare Candy ignore its own limit (a `Restrict`: id1144, id1815). Grand Tree's
//! limits are liftable (id2327) but Grand Tree is a Stadium too: the two are
//! never in play together.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LushForest",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::Permit(PermitSpec {
                for_: EventPred::All(&[
                    EventPred::Kind(EventKind::Evolve),
                    EventPred::Slot(SlotPred::TypeIs(ct::GRASS)),
                    EventPred::Card(Pred::PokemonType(ct::GRASS)),
                ]),
                lifts: &[Limit::BaseEnteredThisTurn],
                while_: &[],
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
