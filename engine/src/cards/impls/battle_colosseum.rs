//! Battle Cage ("Battle Colosseum M2", PFL, stadium): prevent all damage
//! counters from being placed on Benched Pokémon (both yours and your
//! opponent's) by effects of attacks and Abilities from the opponent's
//! Pokémon.
//!
//! Twinleaf: while this is the stadium in play,
//! * every MoveDamageCountersEffect whose player is the non-active player is
//!   prevented (no bench / source check, no stadium-block probe);
//! * a PutCountersEffect (attack) or PlaceDamageCountersEffect (whose source
//!   card is still in play) on a Benched Pokémon from its owner's opponent
//!   is prevented unless the stadium effect is blocked for that target.
//!   Using the stadium is not allowed (CANNOT_USE_STADIUM, Advanced Rulebook B-04).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BattleColosseum",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::BenchCounters }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
