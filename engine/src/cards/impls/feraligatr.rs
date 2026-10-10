//! Feraligatr (TEF): Torrential Heart — once during your turn, you may put 5
//! damage counters on this Pokémon; attacks used by this Pokémon do 120 more
//! damage this turn. Giant Wave — 160; this Pokémon can't use Giant Wave
//! during your next turn.
//!
//! Torrential Heart puts the 5 counters with one PlaceCounters event (the Ability is its cause; the counters aren't damage
//! and can't be prevented by an attack's protection) and sets the once-per-turn marker; while the marker is on this
//! Pokémon, the main damage of any of its attacks gets +120 at the attack stage (before Weakness and Resistance). Giant
//! Wave's "can't use" is an ApplyEffect event on this Pokémon (`Lasting::CannotUseThisAttackNextTurn`).
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
