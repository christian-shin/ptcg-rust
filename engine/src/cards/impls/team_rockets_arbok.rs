//! Team Rocket's Arbok (DRI): Intimidating Glare — while this Pokémon is
//! your Active, your opponent can't play Pokémon with Abilities from their
//! hand (except Team Rocket's Pokémon). Spinning Tail — 30 damage to each of
//! your opponent's Pokémon.
//!
//! Evolving (directly or with Rare Candy) is playing a Pokémon from the hand
//! (rulings id285, id1998, id1133), so the lock declares only `PlayPokemon`.
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
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl { actions: &[LockedAction::PlayPokemon], card: Pred::PrintsAbility, except: Pred::Tag(tag::TEAM_ROCKET), error: "BLOCKED_BY_ABILITY", ..LockDecl::NONE },
        while_: &[LockWhile::Active],
        ability: true,
    }) }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(30), ..EachSlotSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
