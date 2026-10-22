//! Bruxish (SSP 49): Counterattack - if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if this Pokémon is Knocked Out), put 3 damage counters on the Attacking Pokémon. Big Bite - 50; during your
//! opponent's next turn, the Defending Pokémon can't retreat.
//!
//! Counterattack is a trigger over the Damage event of an opponent's attack on this Pokémon while it is the Active Pokémon
//! (`DamagedActive`, id1992): 3 damage counters on the Attacking Pokémon wherever it is now, placed by the Ability (a
//! PlaceCounters event, not damage; Turtonator POR 17, Maractus JTG). Big Bite arms the `CANT_RETREAT` lock on the Defending
//! Pokémon through the opponent's next turn (Maractus JTG).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bruxish",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::CausePokemon), counters: Num::Lit(3) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
