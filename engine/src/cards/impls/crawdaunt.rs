//! Crawdaunt (MEG 85): Vise Grip - 30. Cutting Riposte - 130; if this Pokémon has any damage counters on it, this attack can be used
//! for [D].
//!
//! Cutting Riposte's cost is set to [D] while this Pokémon has damage counters (`CostChange::SetCost`, for this Pokémon's own attack
//! only; Conkeldurr TWM 105's Gutsy Swing, Azumarill SSP's Glistening Bubbles): a card rule of the attack, not an Ability.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Crawdaunt",
    passives: &[Passive {
        origin: RuleSource::CardRule,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::SetCost(&[ct::DARK]),
            attack: Some(1),
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::Damaged]),
            side: Side::Owner,
            guard: Cond::True,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
