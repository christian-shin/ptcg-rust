//! Eevee (SSP / PRE): Boosted Evolution — as long as this Pokémon is in the
//! Active Spot, it can evolve during your first turn or the turn you play it.
//! Reckless Charge — 30; this Pokémon also does 10 damage to itself.
//!
//! Events batch 2: a `Permit` on its own evolving (`This(Role::Base)`) by the
//! rule (from the hand), lifting the first-turn and came-into-play-this-turn
//! limits while it is the Active Pokémon (also after Strange Timepiece devolved
//! it: id2327). Rare Candy and Grand Tree keep their own restrictions
//! (id1144, id1815).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eevee@SSP|PRE",
    // Boosted Evolution: as long as this Pokémon is in the Active Spot, it can evolve during your
    // first turn or the turn you play it.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Permit(PermitSpec {
            for_: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::Path(EvolvePath::Rule), EventPred::This(Role::Base)]),
            lifts: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn],
            while_: &[LockWhile::Active],
        }),
    }],
    // Reckless Charge: this Pokémon also does 10 damage to itself.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(10))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
