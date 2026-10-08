//! Triggers (vocabulary v1 "Triggers"): an event, then steps. Each event
//! declares the effect kinds it reacts to (`kinds`) and recognizes its
//! effect in `fires`; the steps run as the card's trigger program.

use super::*;
use crate::effects::{mask, EffId, Effect, KindMask};
use crate::game::{Game, R};
use crate::types::GamePhase;
use crate::list::*;

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
/// This card's Pokémon moved from the Bench to the Active Spot during its owner's turn.
pub struct OnMovedSpec {}
/// This Energy card is being attached (before it is attached); the Pokémon is in the slot register.
pub struct OnAttachSpec {}
/// Whose Pokémon is Knocked Out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KoWhich {
    /// The Active Pokémon of the opponent of this card's owner (this card's Pokémon in play, Ability working).
    OppActive,
}
pub struct OnKnockOutSpec {
    pub which: KoWhich,
}
/// The Pokémon this Energy is attached to (its Active) was damaged by an attack of an opponent's Pokémon (even if
/// Knocked Out); the steps run once the attack's own text is done, with the Attacking Pokémon in the slot register.
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
        Event::OnMoved(_) => mask(&[k::MOVED_TO_ACTIVE]),
        Event::OnAttach(_) => mask(&[k::ATTACH_ENERGY]),
        Event::OnKnockOut(_) => mask(&[k::KNOCK_OUT]),
        Event::OnDamagedByAttack(_) => mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]),
        _ => KindMask::EMPTY,
    }
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player.
pub(crate) fn fires(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> R<Option<usize>> {
    match &t.event {
        Event::OnEndTurn(w) => {
            let Effect::EndTurn { p } = *g.e(e) else { return Ok(None) };
            let p = p as usize;
            let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
            let ok = match w.whose {
                Turn::Owner => p == owner,
                Turn::Opp => p != owner,
                Turn::Any => true,
            };
            Ok(ok.then_some(owner))
        }
        Event::OnMoved(_) => {
            let Effect::MovedToActive { p, card } = *g.e(e) else { return Ok(None) };
            let p = p as usize;
            Ok((card == me && g.st.active_player as usize == p && g.st.players[p].moved_to_active_this_turn.contains(&me)).then_some(p))
        }
        Event::OnAttach(_) => {
            let Effect::AttachEnergy { p, card, target } = *g.e(e) else { return Ok(None) };
            if card != me {
                return Ok(None);
            }
            let at = passive::Located { owner: p as usize, held: Some(target) };
            if passive::blocked(g, me, t.origin, at, Some(target)) {
                return Ok(None);
            }
            Ok(Some(p as usize))
        }
        Event::OnKnockOut(k) => {
            let Effect::KnockOut { p, target, .. } = *g.e(e) else { return Ok(None) };
            let owner = p as usize;
            match k.which {
                KoWhich::OppActive => {
                    if target.s != g.st.players[owner].active {
                        return Ok(None);
                    }
                    let Some(at) = passive::locate(g, me, t.origin) else { return Ok(None) };
                    if at.owner == owner || passive::blocked(g, me, t.origin, at, None) {
                        return Ok(None);
                    }
                    Ok(Some(at.owner))
                }
            }
        }
        Event::OnDamagedByAttack(_) => match *g.e(e) {
            // The damage records the trigger; it resolves after the attack's own text.
            Effect::AfterDamage { b, damage } => {
                let t = b.target;
                if !g.st.slot(t.p as usize, t.s).cards.contains(me) || g.st.phase != GamePhase::Attack {
                    return Ok(None);
                }
                if t.p == b.player || g.st.players[t.p as usize].active != t.s {
                    return Ok(None);
                }
                g.attack_trigger(b, damage, me, None, false)?;
                Ok(None)
            }
            Effect::AttackTrigger { p, card, target, source_in_play, retaliate: None, .. } if card == me => {
                if !g.st.slot(target.p as usize, target.s).cards.contains(me) {
                    return Ok(None);
                }
                let at = passive::Located { owner: p as usize, held: Some(target) };
                if passive::blocked(g, me, t.origin, at, Some(target)) || !source_in_play {
                    return Ok(None);
                }
                Ok(Some(p as usize))
            }
            _ => Ok(None),
        },
        _ => unimplemented!("spec trigger not implemented yet (trigger.rs)"),
    }
}

/// Bind the program's registers to what the event is about: the slot register holds the Pokémon an Energy is
/// attached to, or the Attacking Pokémon.
pub(crate) fn bind(g: &Game, e: EffId, t: &Trigger, f: &mut run::Frame) {
    let enc = |s: crate::effects::SlotRef| s.p << 4 | s.s;
    match (&t.event, *g.e(e)) {
        (Event::OnAttach(_), Effect::AttachEnergy { target, .. }) => f.slot = enc(target),
        (Event::OnDamagedByAttack(_), Effect::AttackTrigger { source, .. }) => f.slot = enc(source),
        _ => {}
    }
}

/// The effect stays alive while the trigger's program is suspended (a Knock Out waits for a coin flip).
pub(crate) fn retains(t: &Trigger) -> bool {
    matches!(t.event, Event::OnKnockOut(_))
}
