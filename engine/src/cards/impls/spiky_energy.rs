//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: the block check is made for the attacking player; the counters
//! are a PutCountersEffect on the attacker.
//!
//! Fixed (phase 4b, R7F-15): it reacted to DealDamageEffect (before any damage
//! is put, whatever its amount), so it also fired when the damage was
//! prevented (Crustle's Mysterious Rock Inn, ...) or reduced to 0 although the
//! Pokémon was not damaged. It now reacts to AfterDamageEffect, like Punk
//! Helmet and Lucky Helmet (text: "is damaged by an attack"; rulings 1646,
//! 1839: it stacks and works wherever the Pokémon end up).
//!
//! Step 7 of the attack flow chart (F1): the damage records the trigger (`Game::attack_trigger`) and it resolves
//! after every effect of the attack's own text and its prompts (AttackTrigger): it needs this card to be still
//! attached to the damaged Pokémon (an attack that discards it stops it, ruling 1649), the Special Energy not
//! blocked, and the Attacking Pokémon still in play, wherever it is (rulings 530, 1839).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpikyEnergy",
    // When the Pokémon this is attached to, in the Active Spot, is damaged by an attack from your opponent's Pokémon
    // (even if Knocked Out), put 2 damage counters on the Attacking Pokémon.
    triggers: &[Trigger {
        origin: RuleSource::Energy,
        event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: true, removes_attacker_energy: false, attacker_required: true }),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(2), cause: CounterCause::Attack }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
