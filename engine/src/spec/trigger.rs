//! Triggers (vocabulary v1 "Triggers"): an event, then steps. Each event
//! declares the effect kinds it reacts to (`kinds`) and recognizes its
//! effect in `fires`; the steps run as the card's trigger program.

use super::*;
use crate::effects::{mask, EffId, Effect, KindMask};
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

/// How a Pokémon came into play.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterMethod {
    /// "When you play this Pokémon from your hand to evolve" (any Evolve effect of this card,
    /// Rare Candy included).
    Evolve,
}

pub struct OnEnterPlaySpec {
    pub method: EnterMethod,
}
pub struct OnMovedSpec {}
pub struct OnAttachSpec {}
pub struct OnKnockOutSpec {}
pub struct OnDamagedByAttackSpec {}
pub struct OnCheckupSpec {}
pub struct OnDiscardedSpec {}
pub struct OnAfterAttackTriggersSpec {}

/// Whose turn ending fires the trigger, relative to the card's owner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Turn {
    Owner,
    Opp,
    Any,
}

/// The end of a turn (any copy of the card, in any zone, as for the markers
/// it clears).
pub struct OnEndTurnSpec {
    pub whose: Turn,
}

/// The effect kinds an event reacts to.
pub const fn event_kinds(e: &Event) -> KindMask {
    use crate::effects::k;
    match e {
        Event::OnEndTurn(_) => mask(&[k::END_TURN]),
        Event::OnEnterPlay(_) => mask(&[k::EVOLVE]),
        _ => KindMask::EMPTY,
    }
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player.
pub(crate) fn fires(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> Option<usize> {
    match &t.event {
        Event::OnEndTurn(w) => {
            let Effect::EndTurn { p } = *g.e(e) else { return None };
            let p = p as usize;
            let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
            let ok = match w.whose {
                Turn::Owner => p == owner,
                Turn::Opp => p != owner,
                Turn::Any => true,
            };
            ok.then_some(owner)
        }
        Event::OnEnterPlay(w) => {
            let p = match (w.method, *g.e(e)) {
                (EnterMethod::Evolve, Effect::Evolve { p, card, .. }) if card == me => p as usize,
                _ => return None,
            };
            if t.origin == RuleSource::Ability && crate::prefabs::is_ability_blocked(g, p, me, None) {
                return None;
            }
            Some(p)
        }
        _ => unimplemented!("spec trigger not implemented yet (trigger.rs)"),
    }
}
