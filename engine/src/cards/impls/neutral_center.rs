//! Neutralization Zone (SFA, ACE SPEC stadium): prevent all damage done to Pokémon without a Rule Box by attacks from the
//! opponent's Pokémon ex and Pokémon V (the printed phrase: `Pred::PokemonEx`, `Pred::PokemonV`, not "has a Rule Box";
//! the pool holds only ex); this card can't be put into hand or deck from the discard pile.
//!
//! A `Prevent` over `Kind(Damage)` whose subject has no Rule Box and whose cause is an attack of a Pokémon ex or Pokémon V
//! of the other player (`Actor(NotEventOwner)`), read at step 6 of the damage calculation (APR C-16), for both players'
//! Pokémon; effects of those attacks still happen (id2034: a Stadium discard chosen by the attack still happens).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NeutralizationZone",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::Not(&SlotPred::RuleBox),
                EventPred::All(&[
                    EventPred::Kind(EventKind::Damage),
                    EventPred::Actor(Party::NotEventOwner),
                    EventPred::Cause(CausePred::All(&[CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Pokemon(SlotPred::Top(Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV])))])),
                ]),
            )),
        },
        // "This card can't be put into your hand or deck from the discard pile": a lock over its PutIntoHand / PutIntoDeck.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::NOT_FROM_DISCARD_TO_HAND_OR_DECK) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
