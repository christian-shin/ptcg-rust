//! Steven's Carbink (DRI 86): Stone Palace - as long as this Pokémon is on your Bench, all of your Steven's Pokémon take 30 less
//! damage from attacks from your opponent's Pokémon (after applying Weakness and Resistance). The effect of Stone Palace doesn't
//! stack. Magical Shot - 80.
//!
//! An Ability `DamageTaken` over the owner's Pokémon that carry the Steven's tag, guarded by this Pokémon being on the Bench
//! (an Ability lock turns it off), never reducing the owner's own attacks' damage, non-stacking per Ability name
//! (`NonStack::SameAbility`, Bouffalant SCR 119's Curly Wall).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensCarbink",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec {
            amount: 30,
            subject: SlotPred::Tag(crate::types::tag::STEVENS),
            side: Side::Owner,
            guard: Cond::Slot(SlotExpr::This, SlotPred::IsBench),
            nonstacking: Some(NonStack::SameAbility),
            ..DamageTakenSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
