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
    /// A rules event matching the predicate (events batch 2: EnterPlay, Evolve, Devolve, Swap; batch 3: Attach,
    /// MoveEnergy, MoveTool; batch 4: the condition, healing and coin events; batch 5: ChangeActive): the steps run
    /// once the event is done (after its consequences), in the propagation order of the declaring cards
    /// (`run::after_event`). The program runs for the event's owner when the declaring card is a Stadium
    /// ("that player"), else for the declaring card's owner; the event's spot is the picked slot.
    On(super::event::EventPred),
    OnKnockOut(OnKnockOutSpec),
    OnDamagedByAttack(OnDamagedByAttackSpec),
    OnCheckup(OnCheckupSpec),
    OnEndTurn(OnEndTurnSpec),
    OnDiscarded(OnDiscardedSpec),
    OnAfterAttackTriggers(OnAfterAttackTriggersSpec),
    /// A rule no event expresses (Backtrack Badge): the card names the effect kinds it reacts to and
    /// decides in its own function whether it fires (returning the program's player).
    Custom(CustomEventSpec),
}

pub struct CustomEventSpec {
    pub kinds: &'static [u32],
    pub fires: fn(&mut Game, CardId, EffId) -> Option<usize>,
}

/// Whose Knock Out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KoWhich {
    /// This Pokémon, in the Active Spot, is Knocked Out by damage from an attack of the opponent's
    /// Pokémon. The Attacking Pokémon's slot is the program's picked slot (`SlotExpr::Picked`).
    ThisByAttack,
    /// The Active Pokémon of the opponent of this card's owner (this card's Pokémon in play, Ability
    /// working). The effect stays alive while the program is suspended (a Knock Out waits for a coin flip).
    OppActive,
}
pub struct OnKnockOutSpec {
    pub which: KoWhich,
}
/// The Pokémon this card is part of (an Ability) or attached to (a Tool), in the Active Spot, is damaged
/// by an attack of the opponent's Pokémon (even if Knocked Out). The trigger resolves after the attack's
/// own effects (step 7 of the attack flow), in the attack phase, unless the card's lock is on; the
/// Attacking Pokémon's slot is the program's picked slot (`SlotExpr::Picked`).
pub struct OnDamagedByAttackSpec {
    /// The program runs for the attacking player (otherwise for the damaged Pokémon's owner).
    pub as_attacker: bool,
    /// The effect may take an Energy off the Attacking Pokémon (Handheld Fan): it resolves before the
    /// effects that could be blocked by it.
    pub removes_attacker_energy: bool,
    /// It runs only while the Attacking Pokémon is still in play.
    pub attacker_required: bool,
}
impl OnDamagedByAttackSpec {
    pub const DEFAULT: OnDamagedByAttackSpec = OnDamagedByAttackSpec { as_attacker: false, removes_attacker_energy: false, attacker_required: true };
}
/// Pokémon Checkup (between turns); the program's player is the player being checked. An Ability fires for each
/// Pokémon in play that has it, in its owner's part of the Checkup, while it works (Froslass's Freezing Shroud: each
/// Froslass applies it separately, id2302); a card rule or a Trainer's effect fires for every copy of the card in any
/// zone (it clears its own markers).
pub struct OnCheckupSpec {}
/// This card is discarded by an effect of an attack of the Pokémon it is attached to (that
/// player's Active Pokémon).
pub struct OnDiscardedSpec {}
/// The attack's after-attack triggers of the player run (step 7 window).
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
        Event::On(p) => p.effect_kinds(),
        Event::OnKnockOut(_) => mask(&[k::KNOCK_OUT]),
        Event::OnDamagedByAttack(_) => mask(&[k::DAMAGE, k::ATTACK_TRIGGER]),
        Event::OnDiscarded(_) => mask(&[k::DISCARD_CARDS]),
        Event::OnCheckup(_) => mask(&[k::BETWEEN_TURNS]),
        Event::Custom(c) => mask(c.kinds),
        Event::OnAfterAttackTriggers(_) => mask(&[k::AFTER_ATTACK_TRIGGERS]),
    }
}

/// Does effect `e` fire trigger `t` of card `me`? Returns the program's player and the event's slot
/// (the program's picked slot, or NONE).
pub(crate) fn fires(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> Option<(usize, u8)> {
    fires_in(g, me, e, t).map(|(p, slot)| (p, slot.unwrap_or(super::run::NONE)))
}

/// The program's player and, for events about a Pokémon, the slot (`p << 4 | slot`) it works on.
fn fires_in(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> Option<(usize, Option<u8>)> {
    match &t.event {
        // Run after the event is done (`run::after_event`, `fires_on`).
        Event::On(_) => None,
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
        Event::OnKnockOut(OnKnockOutSpec { which: KoWhich::OppActive }) => {
            let Effect::KnockOut { p, target, .. } = *g.e(e) else { return None };
            let owner = p as usize;
            if target.s != g.st.players[owner].active {
                return None;
            }
            let at = super::passive::locate(g, me, t.origin)?;
            if at.owner == owner || super::passive::blocked(g, me, t.origin, at, None) {
                return None;
            }
            Some((at.owner, None))
        }
        Event::OnKnockOut(OnKnockOutSpec { which: KoWhich::ThisByAttack }) => {
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
        Event::OnDamagedByAttack(d) => {
            // Where the card is: part of the Pokémon (an Ability) or attached to it (a Tool).
            let holds = |g: &Game, target: crate::effects::SlotRef| match t.origin {
                RuleSource::Tool => g.st.slot(target.p as usize, target.s).tools.contains(me),
                _ => g.st.slot(target.p as usize, target.s).cards.contains(me),
            };
            match *g.e(e) {
                Effect::Damage { b, amount: damage, .. } => {
                    // Record the step 7 trigger; it resolves later as an AttackTrigger effect.
                    let t0 = b.target;
                    if holds(g, t0) && damage > 0 && b.player != t0.p && g.st.players[t0.p as usize].active == t0.s {
                        let _ = g.attack_trigger(b, damage, me, None, d.removes_attacker_energy);
                    }
                    None
                }
                Effect::AttackTrigger { p, opp, card, target, source, source_in_play, retaliate: None, .. } if card == me => {
                    if !holds(g, target) {
                        return None;
                    }
                    // The lock probe is made for the attacking player.
                    let at = super::passive::Located { owner: p as usize, held: Some(target) };
                    if super::passive::blocked(g, me, t.origin, at, Some(target)) || g.st.phase != crate::types::GamePhase::Attack || (d.attacker_required && !source_in_play) {
                        return None;
                    }
                    Some((if d.as_attacker { p as usize } else { opp as usize }, Some(source.p << 4 | source.s)))
                }
                _ => None,
            }
        }
        Event::OnDiscarded(_) => {
            let Effect::DiscardCards { b, ref cards } = *g.e(e) else { return None };
            let pu = b.player as usize;
            let (sp, ss) = (b.source.p as usize, b.source.s);
            if !(cards.contains(&me) && g.st.slot(sp, ss).cards.contains(me) && g.st.players[pu].active == ss && sp == pu) {
                return None;
            }
            if t.origin == RuleSource::Energy && crate::prefabs::is_special_energy_blocked(g, pu, me, b.source, false) {
                return None;
            }
            // The Pokémon it was attached to is the event's slot.
            Some((pu, Some(b.source.p << 4 | b.source.s)))
        }
        Event::Custom(c) => (c.fires)(g, me, e).map(|p| (p, None)),
        Event::OnCheckup(_) => match *g.e(e) {
            Effect::BetweenTurns { p, .. } if g.st.phase == crate::types::GamePhase::BetweenTurns => {
                let p = p as usize;
                if t.origin == RuleSource::Ability {
                    let at = super::passive::locate(g, me, t.origin)?;
                    if at.owner != p || super::passive::blocked(g, me, t.origin, at, None) {
                        return None;
                    }
                }
                Some((p, None))
            }
            _ => None,
        },
        Event::OnAfterAttackTriggers(_) => match *g.e(e) {
            Effect::AfterAttackTriggers { p, .. } => Some((p as usize, None)),
            _ => None,
        },
    }
}

/// The event an effect carries, as predicates read it (`None` for an effect that carries no event yet).
pub fn event_view(g: &Game, e: EffId) -> Option<super::event::EventView> {
    use super::event::{EventKind, EventView, RulesZone};
    let turn = super::event::whose_turn(g);
    Some(match *g.e(e) {
        Effect::EnterPlay { p, card, target, source, mode, cause, .. } => EventView { source: Some(source), mode: Some(mode), card: Some(card), slot: Some(target), ..EventView::new(EventKind::EnterPlay, cause, p, turn) },
        Effect::Evolve { p, card, base, target, source, path, cause, base_entered_this_turn, owner_first_turn, .. } => EventView { source: Some(source), path: Some(path), card: Some(card), base: Some(base), slot: Some(target), base_entered_this_turn, owner_first_turn, ..EventView::new(EventKind::Evolve, cause, p, turn) },
        Effect::Devolve { p, target, ref removed, cause, .. } => EventView { source: Some(RulesZone::InPlay), card: g.st.slot_pokemon(target.p as usize, target.s), base: removed.get(0).copied(), slot: Some(target), ..EventView::new(EventKind::Devolve, cause, p, turn) },
        Effect::Swap { p, target, old, new, source, cause } => EventView { source: Some(source), card: Some(new), base: Some(old), slot: Some(target), ..EventView::new(EventKind::Swap, cause, p, turn) },
        Effect::Attach { p, card, target, source, manual, cause, .. } => EventView { source: Some(source), manual, card: Some(card), slot: Some(target), ..EventView::new(EventKind::Attach, cause, p, turn) },
        Effect::MoveEnergy { p, card, to, cause, .. } => EventView { source: Some(RulesZone::InPlay), card: Some(card), slot: Some(to), ..EventView::new(EventKind::MoveEnergy, cause, p, turn) },
        Effect::MoveTool { p, card, to, cause, .. } => EventView { source: Some(RulesZone::InPlay), card: Some(card), slot: Some(to), ..EventView::new(EventKind::MoveTool, cause, p, turn) },
        Effect::GainCondition { target, condition, cause, .. } => crate::engine::condition::condition_view(g, EventKind::GainCondition, target, condition, cause),
        Effect::RemoveCondition { target, condition, cause, .. } => crate::engine::condition::condition_view(g, EventKind::RemoveCondition, target, condition, cause),
        Effect::Heal { target, damage, cause, .. } => crate::engine::condition::heal_view(g, target, damage, cause),
        Effect::CoinFlip { p, purpose, heads, cause } => crate::engine::condition::coin_view(g, p as usize, purpose, heads, cause),
        Effect::ChangeActive { p, from, to, change, cause } => crate::engine::change_active::effect_view(g, p, from, to, change, cause),
        Effect::PlaceCounters { target, amount, cause, .. } => crate::engine::damage::counters_view(g, target, amount, cause),
        Effect::Damage { b, amount, .. } => crate::engine::damage::damage_view(g, &b, amount),
        // The whole action (its pairs are in the effect); a trigger over one end would read `end` / `slot` per pair.
        Effect::MoveCounters { p, cause, .. } => EventView::new(EventKind::MoveCounters, cause, p, turn),
        _ => return None,
    })
}

/// Does the event of effect `e` fire the `Event::On` trigger `t` of card `me` (after the event is done)? The
/// declaring card must be in place for its origin and not blocked there (a Pokémon's Ability, the Stadium in
/// play, ...). Returns the program's player (the event's owner for a Stadium, else the card's owner) and the
/// event's spot as the picked slot.
pub(crate) fn fires_on(g: &mut Game, me: CardId, e: EffId, t: &Trigger) -> crate::game::R<Option<(usize, u8)>> {
    let Event::On(pred) = &t.event else { return Ok(None) };
    let Some(v) = event_view(g, e) else { return Ok(None) };
    let Some(at) = super::passive::locate(g, me, t.origin) else { return Ok(None) };
    if !pred.eval(g, me, &v)? {
        return Ok(None);
    }
    if super::passive::blocked(g, me, t.origin, at, v.slot) {
        return Ok(None);
    }
    let p = if t.origin == RuleSource::Stadium { v.owner as usize } else { g.st.owner(me) };
    let slot = v.slot.map_or(super::run::NONE, |s| s.p << 4 | s.s);
    Ok(Some((p, slot)))
}

/// The effect stays alive while the trigger's program is suspended (a Knock Out of the opponent's Active
/// waits for a coin flip).
pub(crate) fn retains(t: &Trigger) -> bool {
    matches!(t.event, Event::OnKnockOut(OnKnockOutSpec { which: KoWhich::OppActive }))
}
