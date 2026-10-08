//! Triggers (vocabulary v1 "Triggers"): an event, then steps. Each event
//! declares the effect kinds it reacts to (`kinds`) and recognizes its
//! effect in `fires`; the steps run as the card's trigger program.

use super::*;
use crate::effects::{mask, EffId, Effect, KindMask};
use crate::game::Game;
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

/// This card enters play (S3 agent 3).
pub struct OnEnterPlaySpec {
    pub how: EnterBy,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterBy {
    /// Played from the hand as a Pokémon (PlayPokemon), not by evolving.
    PlayedFromHand,
    /// Evolved into, from the hand or by an effect (Evolve), Rare Candy included.
    Evolved,
}
/// This Pokémon moved from the Active Spot to the Bench during its owner's turn.
pub struct OnMovedSpec {}
/// This Energy card is attached to a Pokémon, from any zone.
pub struct OnAttachSpec {}
/// This Pokémon, in the Active Spot, is Knocked Out by damage from an attack of the opponent's
/// Pokémon. The Attacking Pokémon's slot is the program's picked slot (`SlotExpr::Picked`).
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
        Event::OnEnterPlay(s) => match s.how {
            EnterBy::PlayedFromHand => mask(&[k::PLAY_POKEMON]),
            EnterBy::Evolved => mask(&[k::EVOLVE]),
        },
        Event::OnKnockOut(_) => mask(&[k::KNOCK_OUT]),
        Event::OnAttach(_) => mask(&[k::ATTACH_ENERGY]),
        Event::OnMoved(_) => mask(&[k::MOVED_FROM_ACTIVE_TO_BENCH]),
        _ => KindMask::EMPTY,
    }
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player.
pub(crate) fn fires(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> Option<(usize, u8)> {
    const NONE: u8 = 0xFF;
    fires_in(g, me, e, t).map(|(p, slot)| (p, slot.unwrap_or(NONE)))
}

/// The program's player and, for events about a Pokémon, the slot (`p << 4 | slot`) it works on.
fn fires_in(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> Option<(usize, Option<u8>)> {
    match &t.event {
        Event::OnEndTurn(w) => {
            let Effect::EndTurn { p } = *g.e(e) else { return None };
            let p = p as usize;
            let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
            // A Tool or an Energy reacts only while attached, unless its lock probe fails.
            if matches!(t.origin, RuleSource::Tool | RuleSource::Energy) {
                let at = super::passive::locate(g, me, t.origin)?;
                if super::passive::blocked(g, me, t.origin, at, at.held) {
                    return None;
                }
            }
            let ok = match w.whose {
                Turn::Owner => p == owner,
                Turn::Opp => p != owner,
                Turn::Any => true,
            };
            ok.then_some((owner, None))
        }
        Event::OnEnterPlay(s) => match (s.how, *g.e(e)) {
            (EnterBy::PlayedFromHand, Effect::PlayPokemon { p, card, .. }) | (EnterBy::Evolved, Effect::Evolve { p, card, .. }) if card == me => {
                let p = p as usize;
                if super::passive::blocked(g, me, t.origin, super::passive::Located { owner: p, held: None }, None) {
                    return None;
                }
                Some((p, None))
            }
            _ => None,
        },
        Event::OnAttach(_) => {
            let Effect::AttachEnergy { p, card, target } = *g.e(e) else { return None };
            if card != me {
                return None;
            }
            let at = super::passive::Located { owner: p as usize, held: Some(target) };
            if super::passive::blocked(g, me, t.origin, at, Some(target)) {
                return None;
            }
            Some((p as usize, None))
        }
        Event::OnKnockOut(_) => {
            let Effect::KnockOut { p, target, .. } = *g.e(e) else { return None };
            let (tp, ts) = (target.p as usize, target.s);
            if !g.st.slot(tp, ts).cards.contains(me) || g.prevented(e) || g.st.slot_pokemon(tp, ts) != Some(me) {
                return None;
            }
            let at = super::passive::Located { owner: p as usize, held: None };
            if super::passive::blocked(g, me, t.origin, at, None) {
                return None;
            }
            let (_, Some(src)) = g.attack_that_damaged_knocked_out(tp, target)? else { return None };
            // The program works on the Attacking Pokémon.
            Some((p as usize, Some(src.p << 4 | src.s)))
        }
        Event::OnMoved(_) => {
            let Effect::MovedFromActiveToBench { p, card } = *g.e(e) else { return None };
            let p = p as usize;
            if card != me || g.st.active_player as usize != p || !g.st.players[p].moved_from_active_to_bench_this_turn.contains(&me) {
                return None;
            }
            if super::passive::blocked(g, me, t.origin, super::passive::Located { owner: p, held: None }, None) {
                return None;
            }
            Some((p, None))
        }
        _ => unimplemented!("spec trigger not implemented yet (trigger.rs)"),
    }
}
