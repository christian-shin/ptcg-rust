//! Decidueye ex (POR / M3): Sniper's Eye - if your opponent has exactly 4 cards in their hand, ignore all [C] Energy in the
//! costs of attacks used by this Pokémon. Crushing Arrow - 240; discard an Energy from your opponent's Active Pokémon.
//!
//! Sniper's Eye is an Ability passive (an Ability lock stops it; it isn't activated) that strips the [C] of the attack cost
//! while the opponent's hand holds 4 cards. Crushing Arrow's discard is an effect of the attack after the damage: a LeavePlay
//! (to the discard pile) of the Energy the attacker chooses, caused by this attack, so Mist Energy and Hide 'n' Sneak refuse
//! it (id2393). With no Energy on the Defending Pokémon nothing is asked.
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
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Register(0), ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
