//! Handheld Fan (TWM, tool): whenever the Active Pokémon this card is
//! attached to takes damage from an opponent's attack, move an Energy from
//! the attacking Pokémon to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: the lock check is a bare ToolEffect for the Fan's owner; the
//! prompt (answered by the Fan owner) moves an Energy from the attacking
//! Pokémon to the attacker's Bench ("your opponent's Benched Pokémon" from
//! the Fan owner's view).
//!
//! Fixed (phase 4b, W4): the "has Bench" check looked at the Fan owner's own
//! Bench (now the attacker's), the ToolEffect probe used the attacking player
//! (now the owner), and the move could be skipped (min 0): it now needs
//! min 1, and nothing happens when the attacker has no Energy.
//!
//! Fixed (phase 4b, R7F-10; rulings 1625, 1649, 1650, 1651): the prompt opened
//! inside the damage step and moved an Energy off the player's Active. The
//! damage now only arms a HANDY_FAN_MARKER on the attacking Pokémon's slot;
//! the effect resolves in AfterAttackEffect, after everything the attack did
//! (an Energy the attack discards or puts away is no longer there, Boomerang
//! Energy is attached again first, an attacker that left play has nothing to
//! move). The attacker is the Pokémon that used the attack even when it is
//! on the Bench (Alakazam ex's Dimensional Hand); it moves the Energy to one
//! of the attacking player's other Benched Pokémon. The marker is cleared at
//! the end of the turn when no AfterAttackEffect came.
//!
//! Fixed (phase 4b, F1, general step 7 mechanism; rulings 1625, 1649, 1650, 1651): the damage records the trigger
//! (`Game::attack_trigger`) and it resolves after the attack's own effects and the prompts they opened (the
//! AttackTrigger effect). The attacker is the Pokémon that used the attack wherever it is then (it can have been
//! switched to the Bench); it moves the Energy to one of the attacking player's other Benched Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HandyFan",
    triggers: &[
        Trigger { origin: RuleSource::Tool, event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: false, removes_attacker_energy: true }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::ToolBlocked), Cond::AttackerInPlay]), yes: &[Step::new(Op::MoveEnergyFromAttacker(MoveEnergyFromAttackerSpec { msg: "ATTACH_ENERGY_TO_BENCH" }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
