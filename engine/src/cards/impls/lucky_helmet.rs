//! Lucky Helmet (TWM, tool): whenever the Pokémon this card is attached to is your Active Pokémon and is damaged by an
//! opponent's attack (even if that Pokémon is Knocked Out), draw 2 cards.
//!
//! `Event::OnDamagedByAttack` on the Damage event (the Active Spot read when the damage is done, id1992): the trigger
//! resolves at step 7 of the attack flow, after the attack's own effects. The Tool must still be attached and not
//! blocked. It draws even if the Attacking Pokémon switched or left play and even if the damaged Pokémon is Knocked Out
//! (the Knock Out is taken later, at the state check).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LuckyHelmet",
    triggers: &[
        Trigger { origin: RuleSource::Tool, event: Event::OnDamagedByAttack(OnDamagedByAttackSpec { as_attacker: true, removes_attacker_energy: false, attacker_required: false }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::Not(&Cond::ToolBlocked), yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(2)) }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
