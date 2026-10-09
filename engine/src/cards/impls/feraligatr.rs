//! Feraligatr (TEF): Torrential Heart — once during your turn, you may put 5
//! damage counters on this Pokémon; attacks used by this Pokémon do 120 more
//! damage this turn. Giant Wave — 160; this Pokémon can't use Giant Wave
//! during your next turn.
//!
//! Twinleaf: the +120 applies to every AttackEffect whose source list holds
//! this card while the player marker is set (not only Giant Wave). The Ability
//! throws BLOCKED_BY_EFFECT when the marker is already set (it is not a
//! USE_ABILITY_ONCE_PER_TURN call) and adds 50 damage to the slot holding
//! this card without any check for Knock Out. Giant Wave pushes its name onto
//! the player's Active `cannotUseAttacksNextTurnPending` if missing.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Feraligatr",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("TORRENTIAL_HEART_MARKER"),
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::This), counters: Num::Lit(5) })),
        ],
    }],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::DamageDealt(DamageDealtSpec { stage: DamageStage::Attack, amount: 120, attacker: SlotPred::Holder, opp_active_only: false, guard: Cond::HasMarker { who: Who::Me, name: "TORRENTIAL_HEART_MARKER", from: MarkerFrom::This }, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
