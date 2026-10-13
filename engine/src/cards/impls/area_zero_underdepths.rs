//! Area Zero Underdepths (SCR, stadium): each player with any Tera Pokémon in play can have up to 8 Benched Pokémon;
//! otherwise the Bench shrinks back to 5.
//!
//! A `BenchSize` Stadium declaration read wherever the Bench size matters; a Bench over its size is discarded down at the
//! state check, before the Knock Outs are scanned (id2305, JP FAQ Area Zero Underdepths) and before a new Active Pokémon is
//! chosen (id2131). Using the Stadium is not allowed (APR B-04).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AreaZeroUnderdepths",
    passives: &[
        // Automatically active: it can't be announced and used.
        // Each player with any Tera Pokémon in play can have up to 8 Benched Pokémon.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::BenchSize(BenchSizeSpec { size: 8, guard: Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA)) }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
