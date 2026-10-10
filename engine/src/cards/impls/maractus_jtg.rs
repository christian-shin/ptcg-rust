//! Maractus (JTG): Explosive Needle — if this Pokémon is in the Active Spot
//! and is Knocked Out by damage from an attack from your opponent's Pokémon,
//! put 6 damage counters on the Attacking Pokémon. Corner — 20; the
//! Defending Pokémon can't retreat during your opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Maractus@JTG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        // "If this Pokémon is in the Active Spot and is Knocked Out by damage from an attack from your opponent's Pokémon":
        // its KnockOut (while it is still in play: APR D step 2), the Active Spot read when the damage was done (id1992).
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::KnockOut),
            EventPred::This(Role::Card),
            EventPred::DamagedActive,
            EventPred::KoBy(KoBy::AttackDamage),
            EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
        ])),
        // 6 damage counters on the Attacking Pokémon wherever it is now (nothing when it left play), placed by the Ability
        // (not damage): Hide 'n' Sneak and the like prevent them.
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::CausePokemon), counters: Num::Lit(6) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
