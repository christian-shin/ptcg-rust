//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! Rule: a `Prevent` (`PreventWhat::MoveToHandFromOppPlay`, read per card move) on a
//! card going from the opponent's Pokémon in play to that player's hand, whoever
//! makes the move. The effect that tries it still happens otherwise (id2129:
//! Professor Turo's Scenario is played and discarded, the Pokémon and its cards stay;
//! id2370: Ninja Spinner's bonus damage is done, the Water Energy stays attached).
//! B6-OLD -> batch 7: the attached-card half (the Discard / PutIntoHand events).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiloticTWMPool",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::MoveToHandFromOppPlay, ..PreventSpec::NONE }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
