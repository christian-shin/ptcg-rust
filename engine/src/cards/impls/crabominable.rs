//! Crabominable (SCR): Food Prep — attacks used by this Pokémon cost [C] less for each Kofu card in your discard pile. Haymaker —
//! 250; during your next turn, this Pokémon can't use Haymaker.
//!
//! Food Prep is an Ability `AttackCost` reduction over this Pokémon's attacks, one [C] for each Kofu card in the discard pile
//! (summed with other changes and floored at zero, APR D-11, D-12). Haymaker arms a lasting lock on this Pokémon naming
//! Haymaker (`NamedAttack::This`) until the end of your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Crabominable",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Attacks cost [C] less for each Kofu card in your discard pile.
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Trainer, Pred::Name("Kofu")]))),
            subject: SlotPred::IsThisPokemon,
            side: Side::Owner,
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
