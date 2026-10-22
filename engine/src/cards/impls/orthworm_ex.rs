//! Orthworm ex (SCR 110): Pummeling Payback - if this Pokémon is damaged by an attack from your opponent's Pokémon (even if this
//! Pokémon is Knocked Out), put 2 damage counters on the Attacking Pokémon for each [M] Energy attached to this Pokémon. Rock Tomb -
//! 150; during your opponent's next turn, the Defending Pokémon can't retreat.
//!
//! Pummeling Payback is a trigger over the Damage event of an opponent's attack on this Pokémon, in any spot (Turtonator POR's form
//! without the Active Spot): the [M] Energy it provides are counted when the Ability resolves, after the attack's own effects, so
//! one the attack discarded isn't counted (id2012; id2237: Energy attached is Energy provided); the counters are a PlaceCounters event
//! by the Ability on the Attacking Pokémon wherever it is now.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "OrthwormEx",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder)])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec {
            target: SlotTarget::Slot(SlotExpr::CausePokemon),
            counters: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::Provided(ct::METAL)), &Num::Lit(2)),
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
