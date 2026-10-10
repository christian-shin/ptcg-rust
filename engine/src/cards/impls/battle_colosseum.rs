//! Battle Cage ("Battle Colosseum M2", PFL, stadium): prevent all damage
//! counters from being placed on Benched Pokémon (both yours and your
//! opponent's) by effects of attacks and Abilities from the opponent's
//! Pokémon. (Damage from attacks is still taken.)
//!
//! Rule: one `Prevent` over the PlaceCounters event, and over the arriving end of
//! a MoveCounters event, on a Benched Pokémon whose cause is an attack or an
//! Ability of the other player's Pokémon (`Actor(NotEventOwner)`). Counters moved
//! onto such a Pokémon leave their source and vanish (id2257, id79; JP FAQ
//! Battle Cage PFL 85); Battle Cage only stops the placing (id79).
//! Counters placed on the Active Pokémon, and damage to a Benched Pokémon, are
//! not stopped; your own Pokémon's Abilities still place counters on your own
//! Bench (JP FAQ). Using the stadium is not allowed (APR B-04).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BattleColosseum",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        // Damage counters placed on a Benched Pokémon, or moved onto one (they leave their source and vanish: id2257, JP FAQ
        // Battle Cage x3), by an effect of an attack or Ability from the Pokémon of that Pokémon's owner's opponent.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::IsBench,
                EventPred::All(&[
                    EventPred::Any(&[EventPred::Kind(EventKind::PlaceCounters), EventPred::All(&[EventPred::Kind(EventKind::MoveCounters), EventPred::End(MoveEnd::To)])]),
                    EventPred::Cause(CausePred::Any(&[CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Kind(crate::cause::CauseKind::Ability)])),
                    EventPred::Actor(Party::NotEventOwner),
                ]),
            )),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
