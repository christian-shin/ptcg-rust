//! Neutralization Zone (SFA, ACE SPEC stadium): prevent all damage done to
//! Pokémon without a Rule Box by attacks from the opponent's Pokémon with a
//! Rule Box; this card can't be put into hand or deck from the discard pile.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NeutralCenter",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::Not(&SlotPred::RuleBox),
                EventPred::All(&[
                    EventPred::Kind(EventKind::Damage),
                    EventPred::Actor(Party::NotEventOwner),
                    EventPred::Cause(CausePred::All(&[CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Pokemon(SlotPred::RuleBox)])),
                ]),
            )),
        },
        // "This card can't be put into your hand or deck from the discard pile": a lock over its PutIntoHand / PutIntoDeck.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::NOT_FROM_DISCARD_TO_HAND_OR_DECK) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
