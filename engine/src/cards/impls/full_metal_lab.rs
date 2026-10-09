//! Full Metal Lab (TEF, stadium): [M] Pokémon (both yours and your
//! opponent's) take 30 less damage from attacks from the opponent's Pokémon
//! (after applying Weakness and Resistance).
//!
//! Twinleaf: every PutDamageEffect from an attack of the opponent's Pokémon
//! on a [M] Pokémon (CheckPokemonTypeEffect) is reduced by 30 (floored at 0)
//! unless the stadium effect is blocked for the target's owner. The stadium
//! can't be used.
//!
//! Fixed (phase 4b, R2): the reduction also applied to damage a player's own
//! attack put on its own [M] Pokémon (recoil, own Bench damage); the text only
//! reduces damage from attacks of the opponent's Pokémon.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "FullMetalLab",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::TypeIs(ct::METAL), ..DamageTakenSpec::DEFAULT }),
        },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
