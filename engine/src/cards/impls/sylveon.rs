//! Sylveon (PRE): Safeguard - prevent all damage done to this Pokémon by attacks from your opponent's Pokémon ex.
//! Magical Shot - 100.
//!
//! A `Prevent` over `Kind(Damage)` whose cause is an attack of the opponent's Pokémon tagged ex
//! (`CausePred::Card`), read at step 6 of the damage calculation (APR C-16); not while the Ability is blocked, and not
//! against an attack that ignores the effects on the Defending Pokémon (id2095, Demolish). Effects of those attacks
//! still happen.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Sylveon",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Cause(CausePred::Card(Pred::Tag(tag::POKEMON_EX_LOWER)))]))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
