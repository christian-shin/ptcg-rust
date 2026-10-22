//! Shelmet (WHT 8): Stimulated Evolution - if you have Karrablast in play, this Pokémon can evolve during your first turn or the
//! turn you play it. Headbutt Bounce - 10.
//!
//! A `Permit` on its own evolving on any path (the text doesn't say "from your hand"), lifting the first-turn and
//! came-into-play-this-turn limits while a Pokémon named Karrablast is in play (Eevee SSP, Lush Forest; the same form as Karrablast).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ShelmetWHT",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Permit(PermitSpec {
            for_: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Base)]),
            lifts: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn],
            while_: &[LockWhile::Unless(&Cond::Not(&Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("Karrablast"))))],
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
