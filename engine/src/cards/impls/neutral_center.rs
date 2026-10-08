//! Neutralization Zone (SFA, ACE SPEC stadium): prevent all damage done to
//! Pokémon without a Rule Box by attacks from the opponent's Pokémon with a
//! Rule Box; this card can't be put into hand or deck from the discard pile.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NeutralCenter",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::PreventDamage(PreventDamageSpec { subject: SlotPred::Not(&SlotPred::RuleBox), source: SlotPred::RuleBox, ..PreventDamageSpec::DEFAULT }),
        },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::ThisCardFromDiscard }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
