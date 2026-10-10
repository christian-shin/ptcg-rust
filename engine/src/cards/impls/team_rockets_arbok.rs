//! Team Rocket's Arbok (DRI): Potent Glare — as long as this Pokémon is in
//! the Active Spot, your opponent can't play any Pokémon that has an Ability
//! from their hand, except for Team Rocket's Pokémon. Spinning Tail — 30
//! damage to each of your opponent's Pokémon.
//!
//! Evolving (directly or with Rare Candy) is playing a Pokémon from the hand
//! (rulings id285, id1998, id1133), so the lock covers EnterPlay and Evolve from the hand.
//! Spinning Tail's 30 is one Damage event per Pokémon (no Weakness or Resistance for the Benched ones, APR B-08).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsArbok",
    // Intimidating Glare: while this Pokémon is your Active Pokémon, your opponent can't play Pokémon with Abilities
    // from their hand (except Team Rocket's Pokémon).
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl::on(EventPred::All(&[EventPred::Any(&[EventPred::Kind(EventKind::EnterPlay), EventPred::Kind(EventKind::Evolve)]), EventPred::Source(RulesZone::Hand), EventPred::Card(Pred::All(&[Pred::PrintsAbility, Pred::Not(&Pred::Tag(tag::TEAM_ROCKET))]))]), "BLOCKED_BY_ABILITY"),
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
