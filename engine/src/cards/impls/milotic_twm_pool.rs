//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! The whole Pokémon: a `Prevent` over `LeavePlay & Dest(Hand)` of the opponent's Pokémon (`!OnMySide`), whatever
//! causes it and whoever makes the move; the LeavePlay is refused before it happens (no event, no trigger, nothing
//! reset). The effect that tries it still happens otherwise (id2129: Professor Turo's Scenario / Scoop Up Cyclone is
//! played and discarded, the Pokémon and its cards stay). The attached cards: B6-OLD -> batch 7, a `Prevent`
//! (`PreventWhat::MoveToHandFromOppPlay`, read per card move) on a card going from the opponent's Pokémon in play to
//! that player's hand (id2370: Ninja Spinner's bonus damage is done, the Water Energy stays attached); batch 7 makes
//! it a `Prevent` over the PutIntoHand event.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiloticTWMPool",
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::Not(&SlotPred::OnMySide),
                EventPred::All(&[EventPred::Kind(EventKind::LeavePlay), EventPred::Dest(RulesZone::Hand)]),
            )),
        },
        // B6-OLD -> batch 7: the attached-card half.
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::MoveToHandFromOppPlay, ..PreventSpec::NONE }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
