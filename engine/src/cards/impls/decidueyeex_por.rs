//! Decidueye ex (POR / M3): Sniper's Eye — if your opponent has exactly 4
//! cards in their hand, ignore all [C] Energy in the costs of attacks used by
//! this Pokémon. Crushing Arrow — 240; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Fixed (phase 4b, R3): Sniper's Eye now checks Ability locks (IS_ABILITY_BLOCKED
//! for the owner before reading the opponent's hand), is not an activated
//! Ability (`use_when_in_play` false in the card DB), and Crushing Arrow's
//! discard is DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON on a fresh
//! AttackEffect (a DiscardCardsEffect, so Mist Energy can prevent it) instead
//! of a bare MOVE_CARDS.
//!
//! Twinleaf: Sniper's Eye strips every [C] from any CheckAttackCostEffect
//! while this card is the player's Active Pokémon (R7F-11: and sets
//! `ignoreColorless`, so a [C] added later by another effect is ignored too). Crushing Arrow
//! (AFTER_ATTACK) prompts (non-cancellable ChooseCardsPrompt, Energy, over the
//! opponent's Active) only when the Active holds an Energy card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Decidueyeex@POR",
    // Sniper's Eye: if your opponent has exactly 4 cards in their hand, ignore all [C] Energy in the
    // costs of attacks used by this Pokémon.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::IgnoreColorless,
            side: Side::Owner,
            guard: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Eq, Num::Lit(4)),
            ..AttackCostSpec::DEFAULT
        }),
    }],
    // Crushing Arrow: discard an Energy from your opponent's Active Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Opp, Zone::Attached(OPP_ACTIVE)),
                predicate: Pred::Energy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: OPP_ACTIVE, selection: EnergySelection::Register(0) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
