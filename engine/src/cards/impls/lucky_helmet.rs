//! Lucky Helmet (TWM, tool): if the Pokémon this card is attached to is in
//! the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if it is Knocked Out), draw 2 cards.
//!
//! Twinleaf: reacts to AfterDamageEffect on the holder; the tool block probe
//! is a bare ToolEffect for the attacking player and runs first; the draw is
//! MOVE_CARDS(count 2) from the attacked player's deck (no phase check).
//!
//! Step 7 of the attack flow chart (F1): the damage records the trigger and it resolves after the attack's own
//! effects (AttackTrigger): the Tool must still be attached (ruling 1649) and not blocked. It draws even if the
//! Attacking Pokémon switched or left play (ruling 1827).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LuckyHelmet",
    triggers: &[
        Trigger { origin: RuleSource::Tool, event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: true, removes_attacker_energy: false }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::Not(&Cond::ToolBlocked), yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(2)) }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
