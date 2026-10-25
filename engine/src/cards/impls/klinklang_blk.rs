//! Klinklang (BLK 63): Gear Coating - all of your Pokémon that have any [M] Energy attached take 20 less damage from attacks from your
//! opponent's Pokémon (after applying Weakness and Resistance). Hammer In - 120.
//!
//! A `DamageTaken` of 20 over the owner's Pokémon whose Energy provides [M] (Aurorus POR 24's form); the text doesn't say it
//! doesn't stack, so every Klinklang with a working Ability reduces by 20. This is a different card from Klinklang SCR 101.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KlinklangBLK",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 20, subject: SlotPred::Provides(ct::METAL), side: Side::Owner, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
