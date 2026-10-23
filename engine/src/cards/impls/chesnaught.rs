//! Chesnaught (CRI 7): Needly Armor - if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if this Pokémon is Knocked Out), place 3 damage counters on the Attacking Pokémon for each [G] Energy attached to
//! this Pokémon. Impound - 160; during your opponent's next turn, the Defending Pokémon can't retreat.
//!
//! Needly Armor is a trigger over the Damage event of an opponent's attack on this Pokémon while it is the Active Pokémon
//! (`DamagedActive`, id1992), resolved after the attack's own effects (APR E-03): 3 damage counters for each [G] Energy this
//! Pokémon provides (counted in units: a Special Energy that provides [G] and a doubled Energy count, id2347) by the Ability (a
//! PlaceCounters event, not damage) on the Attacking Pokémon wherever it is now (Bruxish SSP 49). Impound arms the lasting
//! `CANT_RETREAT` lock on the Defending Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Chesnaught",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec {
            target: SlotTarget::Slot(SlotExpr::CausePokemon),
            counters: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(3)),
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
