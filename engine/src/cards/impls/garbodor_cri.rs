//! Garbodor (CRI 57): Gloomy Garbage - attacks used by your opponent's Active Pokémon that has a Pokémon Tool attached do
//! 20 less damage (before applying Weakness and Resistance). Sludge Bomb - 100.
//!
//! A `DamageDealt` declaration at the Deal stage of -20 on every attack of the opponent's Active Pokémon while it has a
//! Tool attached, to any Pokémon the attack damages, a Benched one too (id2345), whether or not this Pokémon is Active; an
//! effect on the attacking Pokémon, so Shred doesn't ignore it (id2346), and two copies add up (id2344).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GarbodorCRI",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: -20,
            attacker: SlotPred::All(&[SlotPred::IsActive, SlotPred::AnyTool(Pred::Any)]),
            side: Side::Opponent,
            opp_active_only: false,
            needs_damage: true,
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
