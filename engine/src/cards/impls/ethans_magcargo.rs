//! Ethan's Magcargo (ASC / DRI): Melt Away — if this Pokémon has no Energy attached, it has no Retreat Cost. Lava Burst —
//! 70×; discard up to 5 [R] Energy from this Pokémon. This attack does 70 damage for each card you discarded in this way.
//!
//! The Energy is discarded before the damage (each card a LeavePlay caused by the attack; "up to" is 0 to 5 for an
//! attack, id2301) and the damage counts the cards discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansMagcargo",
    // Melt and Flow: if this Pokémon has no Energy attached, it has no Retreat Cost.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::RetreatCost(RetreatCostSpec {
            change: CostChange::Free,
            subject: SlotPred::All(&[SlotPred::IsThisPokemon, SlotPred::NoEnergyProvided]),
            side: Side::Owner,
            ..RetreatCostSpec::DEFAULT
        }),
    }],
    // Lava Burst: discard up to 5 [R] Energy from this Pokémon, 70 damage for each card discarded.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Prompt { ty: Some(ct::FIRE), min: 0, max: 5, into: Some(0) }, ..DiscardEnergySpec::DEFAULT })),
            Step::before_damage(damage_is(Num::Mul(&Num::RegCount(0), &Num::Lit(70)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
