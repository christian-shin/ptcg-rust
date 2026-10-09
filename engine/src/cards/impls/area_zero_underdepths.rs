//! Area Zero Underdepths (SCR, stadium): each player with any Tera Pokémon in
//! play can have up to 8 Benched Pokémon; otherwise the Bench shrinks back to
//! 5 (handled by the check-state bench size change).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AreaZeroUnderdepths",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        // Each player with any Tera Pokémon in play can have up to 8 Benched Pokémon.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::BenchSize(BenchSizeSpec { size: 8, guard: Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA)) }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
