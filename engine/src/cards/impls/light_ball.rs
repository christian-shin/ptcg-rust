//! Light Ball (ASC 191, Tool): attacks used by the Pikachu ex this card is attached to do 50 more damage to your opponent's
//! Active Pokémon ex (before applying Weakness and Resistance).
//!
//! `DamageDealt` +50 with the attacker bound to the holder named Pikachu ex (`Holder` and `Named`: Pikachu EX is another name,
//! JP FAQ Light Ball) and the target an ex, only when the attack does damage (id675); it works while the Tool does.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "LightBall",
    passives: &[
        Passive { origin: RuleSource::Tool, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 50, attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Named("Pikachu ex")]), target: SlotPred::Tag(tag::POKEMON_EX_LOWER), needs_damage: true, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
