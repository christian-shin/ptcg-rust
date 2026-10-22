//! Karrablast (BLK 9): Stimulated Evolution - if you have Shelmet in play, this Pokémon can evolve during your first turn or
//! the turn you play it. Horn Attack - 10.
//!
//! A `Permit` on its own evolving on any path, lifting the first-turn and came-into-play-this-turn limits while Shelmet is
//! in play (`LockWhile::Unless` over the lack of Shelmet; Eevee SSP, Lush Forest).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KarrablastBLK",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Permit(PermitSpec {
            for_: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Base)]),
            lifts: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn],
            while_: &[LockWhile::Unless(&Cond::Not(&Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("Shelmet"))))],
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
