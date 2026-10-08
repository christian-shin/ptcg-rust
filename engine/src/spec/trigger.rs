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

/// This card is played from the hand: put into play (`evolved: false`: PlayPokemon, onto the Bench) or
/// evolving a Pokémon (`evolved: true`: Evolve, however the card got there, Rare Candy included). The steps
/// run when the effect is reduced, before the card is placed.
pub struct OnEnterPlaySpec {
    pub evolved: bool,
}
pub struct OnMovedSpec {}
/// This Energy card is attached (to any Pokémon, by any effect); the steps run before it is placed.
pub struct OnAttachSpec {}
pub struct OnKnockOutSpec {}
/// The Pokémon this card is part of (an Ability) or attached to (a Tool) is damaged by an attack of the
/// opponent's Pokémon while it is Active (even if it is Knocked Out); the effect resolves in step 7 of the
/// attack, after the attack's own effects. The program's player is the attacking player (`as_attacker`: the
/// lock probes then are the attacker's, as today for Heatran and Lucky Helmet) or the damaged Pokémon's
/// owner. `SlotExpr::Attacker` is the Pokémon that used the attack.
pub struct OnDamagedByAttackSpec {
    pub as_attacker: bool,
    /// The effect may take an Energy off the Attacking Pokémon (Handheld Fan): it resolves before the effects
    /// that could be blocked by it.
    pub removes_attacker_energy: bool,
}
/// Pokémon Checkup (between turns), for every copy of the card in any zone; the program's player is the
/// player being checked.
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
        Event::OnEnterPlay(s) => {
            if s.evolved {
                mask(&[k::EVOLVE])
            } else {
                mask(&[k::PLAY_POKEMON])
            }
        }
        Event::OnAttach(_) => mask(&[k::ATTACH_ENERGY]),
        Event::OnDamagedByAttack(_) => mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]),
        Event::OnCheckup(_) => mask(&[k::BETWEEN_TURNS]),
        _ => KindMask::EMPTY,
    }
}

/// Schedule what a trigger records before its effect resolves: the damage of an attack opens a step 7
/// trigger (`Game::attack_trigger`).
pub(crate) fn prepare(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> crate::game::R {
    if let Event::OnDamagedByAttack(d) = &t.event {
        if let Effect::AfterDamage { b, damage } = *g.e(e) {
            let target = b.target;
            let holds = match t.origin {
                RuleSource::Tool => g.st.slot(target.p as usize, target.s).tools.contains(me),
                _ => g.st.slot(target.p as usize, target.s).cards.contains(me),
            };
            if holds && damage > 0 && b.player != target.p && g.st.players[target.p as usize].active == target.s {
                g.attack_trigger(b, damage, me, None, d.removes_attacker_energy)?;
            }
        }
    }
    Ok(())
}

/// What the program of a trigger knows about its event (`Frame::ctx`).
pub(crate) fn context(g: &Game, e: EffId) -> u16 {
    match *g.e(e) {
        Effect::AttackTrigger { source, source_in_play, .. } => (source.p as u16) << 4 | source.s as u16 | (source_in_play as u16) << 8,
        _ => 0,
    }
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player.
pub(crate) fn fires(g: &Game, me: CardId, e: EffId, t: &Trigger) -> Option<usize> {
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
        Event::OnEnterPlay(s) => match *g.e(e) {
            Effect::PlayPokemon { p, card, .. } if !s.evolved && card == me => Some(p as usize),
            Effect::Evolve { p, card, .. } if s.evolved && card == me => Some(p as usize),
            _ => None,
        },
        Event::OnDamagedByAttack(d) => match *g.e(e) {
            Effect::AttackTrigger { p, opp, card, target, retaliate: None, .. } if card == me => {
                let holds = match t.origin {
                    RuleSource::Tool => g.st.slot(target.p as usize, target.s).tools.contains(me),
                    _ => g.st.slot(target.p as usize, target.s).cards.contains(me),
                };
                (holds && g.st.phase == crate::types::GamePhase::Attack).then_some(if d.as_attacker { p as usize } else { opp as usize })
            }
            _ => None,
        },
        Event::OnAttach(_) => match *g.e(e) {
            Effect::AttachEnergy { p, card, .. } if card == me => Some(p as usize),
            _ => None,
        },
        Event::OnCheckup(_) => match *g.e(e) {
            Effect::BetweenTurns { p, .. } if g.st.phase == crate::types::GamePhase::BetweenTurns => Some(p as usize),
            _ => None,
        },
        _ => unimplemented!("spec trigger not implemented yet (trigger.rs)"),
    }
}
