//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! An `OnDamagedByAttack` trigger over the Damage event: it records only a Damage event of more than 0 that an
//! opponent's attack did to the Active Pokémon this card is attached to, so damage that was prevented (Crustle's
//! Mysterious Rock Inn) or reduced to 0 doesn't count. It stacks (one per copy, ruling 1646) and works wherever the
//! Pokémon end up (ruling 1839).
//!
//! Step 7 of the attack flow chart (F1): the trigger resolves after every effect of the attack's own text and its prompts
//! (`AttackTrigger`). It needs this card to be still attached to the damaged Pokémon (an attack that discards it stops
//! it, ruling 1649), the Special Energy not blocked (the lock is read for the attacking player), and the Attacking
//! Pokémon still in play, wherever it is (rulings 530, 1839). The counters are one PlaceCounters event on the Attacking
//! Pokémon, cause this card's Energy rule.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpikyEnergy",
    // When the Pokémon this is attached to, in the Active Spot, is damaged by an attack from your opponent's Pokémon
    // (even if Knocked Out), put 2 damage counters on the Attacking Pokémon.
    triggers: &[Trigger {
        origin: RuleSource::Energy,
        event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: true, removes_attacker_energy: false, attacker_required: true }),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(2) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
