//! Team Rocket's Arbok (DRI): Intimidating Glare — while this Pokémon is
//! your Active, your opponent can't play Pokémon with Abilities from their
//! hand (except Team Rocket's Pokémon). Spinning Tail — 30 damage to each of
//! your opponent's Pokémon.
//!
//! Twinleaf: any PlayPokemonEffect (bench or evolve) by the player whose
//! opponent has this card as the Active top card throws when the played card
//! has an Ability after CheckPokemonPowersEffect.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsArbok",
    // Intimidating Glare: while this Pokémon is your Active Pokémon, your opponent can't play Pokémon with Abilities
    // from their hand (except Team Rocket's Pokémon).
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::PlayAbilityPokemon(tag::TEAM_ROCKET) }) }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(30), ..EachSlotSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
