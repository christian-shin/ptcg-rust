//! Ethan's Magcargo (DRI 36 / ASC 24): Melt and Flow — if this Pokémon has no
//! Energy attached, it has no Retreat Cost. Lava Burst — 70x; discard up to 5
//! [R] Energy from this Pokémon, 70 damage for each card discarded.
//!
//! Twinleaf: Melt and Flow reacts to CheckRetreatCostEffect when this card is
//! in its player's Active (generic Ability probe, then a CheckProvidedEnergy on
//! the Active: no entries -> `cost = []`). Lava Burst runs a CheckProvidedEnergy
//! (unused), a non-cancellable DiscardEnergyPrompt (Active, Energy cards that
//! don't provide [R] blocked, min 0, max 5), then one DiscardCardsEffect (possibly with
//! no cards) aimed at `player.active` and `damage = 70 * cards`.
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
