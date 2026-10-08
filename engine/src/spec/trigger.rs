//! Triggers (vocabulary v1 "Triggers"): an event, then steps. Each event
//! declares the effect kinds it reacts to (`kinds`) and recognizes its
//! effect in `fires`; the steps run as the card's trigger program.

use super::*;
use crate::effects::{EffId, KindMask};
use crate::game::Game;
use crate::list::CardId;

pub struct Trigger {
    pub origin: RuleSource,
    pub event: Event,
    pub steps: &'static [Step],
}

pub enum Event {
    OnEnterPlay(OnEnterPlaySpec),
    OnMoved(OnMovedSpec),
    OnAttach(OnAttachSpec),
    OnKnockOut(OnKnockOutSpec),
    OnDamagedByAttack(OnDamagedByAttackSpec),
    OnCheckup(OnCheckupSpec),
    OnEndTurn(OnEndTurnSpec),
    OnDiscarded(OnDiscardedSpec),
    OnAfterAttackTriggers(OnAfterAttackTriggersSpec),
}

pub struct OnEnterPlaySpec {}
pub struct OnMovedSpec {}
pub struct OnAttachSpec {}
pub struct OnKnockOutSpec {}
pub struct OnDamagedByAttackSpec {}
pub struct OnCheckupSpec {}
pub struct OnEndTurnSpec {}
pub struct OnDiscardedSpec {}
pub struct OnAfterAttackTriggersSpec {}

/// The effect kinds an event reacts to.
pub const fn event_kinds(_e: &Event) -> KindMask {
    KindMask::EMPTY
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player.
pub(crate) fn fires(_g: &Game, _me: CardId, _e: EffId, _t: &Trigger) -> Option<usize> {
    unimplemented!("spec trigger not implemented yet (trigger.rs)")
}
