//! Battle Cage ("Battle Colosseum M2", PFL, stadium): prevent all damage
//! counters from being placed on Benched Pokémon (both yours and your
//! opponent's) by effects of attacks and Abilities from the opponent's
//! Pokémon.
//!
//! While this is the stadium in play, a PutCountersEffect (attack), a
//! MoveCountersEffect or a PlaceDamageCountersEffect (whose source card is
//! still in play) on a Benched Pokémon from its owner's opponent is prevented
//! unless the stadium effect is blocked for that target. Counters moved onto
//! such a Pokémon leave their source and vanish (ruling 2257).
//!   Using the stadium is not allowed (CANNOT_USE_STADIUM, Advanced Rulebook B-04).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BattleColosseum",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::BenchCounters }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
