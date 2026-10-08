//! Azumarill (SSP): Glistening Bubbles - with a Tera Pokémon in play,
//! Double-Edge costs less. Double-Edge - 230; 50 damage to itself.
//!
//! Twinleaf: on CheckAttackCostEffect for attack 0, when any of the player's
//! in-play Pokémon has the Tera tag and an Ability probe passes, the first
//! [P] of the cost is removed together with the two entries after it
//! (`splice(index, 3)`), so [P][P][P][P] becomes [P] (fixed in phase 4b: the
//! old loop repeated the splice and left an empty cost). The self-damage
//! is a DealDamageEffect targeting the player's Active (not necessarily this
//! Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Azumarill",
    // Glistening Bubbles: with any Tera Pokémon in play, you can use Double-Edge for [P].
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::SetCost(&[ct::PSYCHIC]),
            attack: Some(0),
            side: Side::Owner,
            guard: Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA)),
            ..AttackCostSpec::DEFAULT
        }),
    }],
    // Double-Edge: 50 damage to itself.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(50))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
