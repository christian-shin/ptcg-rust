//! Perilous Jungle (TEF): during Pokémon Checkup, put 2 more damage counters
//! on each Poisoned non-[D] Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PerilousJungle",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::CheckupDamage(CheckupDamageSpec {
                amount: 20,
                victim: SlotPred::Not(&SlotPred::TypeIs(crate::types::ct::DARK)),
                opponent_only: false,
                holder: SlotPred::Any,
                burn: false,
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
