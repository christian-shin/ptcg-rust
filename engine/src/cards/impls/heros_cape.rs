//! Hero's Cape (TEF, ACE SPEC Tool): the Pokémon this card is attached to
//! gets +100 HP.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HerosCape",
    passives: &[Passive { origin: RuleSource::Tool, modifier: Modifier::HpBonus(100) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
