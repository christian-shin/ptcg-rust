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
    OnDamagedByAttack(OnDamagedByAttackSpec),
    OnAfterAttackTriggers(OnAfterAttackTriggersSpec),
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
/// The attack's after-attack triggers of the player run (step 7 window).
pub struct OnAfterAttackTriggersSpec {}

/// The effect kinds an event reacts to.
pub const fn event_kinds(e: &Event) -> KindMask {
    use crate::effects::k;
    match e {
        Event::On(p) => p.effect_kinds(),
        Event::OnDamagedByAttack(_) => mask(&[k::DAMAGE, k::ATTACK_TRIGGER]),
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
        Effect::Devolve { p, target, ref removed, dest, cause, .. } => EventView { source: Some(RulesZone::InPlay), card: g.st.slot_pokemon(target.p as usize, target.s), base: removed.get(0).copied(), slot: Some(target), dest: Some(crate::engine::knockout::zone_of(dest)), ..EventView::new(EventKind::Devolve, cause, p, turn) },
        Effect::Swap { p, target, old, new, source, cause, .. } => EventView { source: Some(source), card: Some(new), base: Some(old), slot: Some(target), ..EventView::new(EventKind::Swap, cause, p, turn) },
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
        Effect::KnockOut { target, ko_by, cause, .. } => crate::engine::knockout::ko_view(g, target, ko_by, cause),
        // The Pokémon as it was when it left (the view is read after the reducer emptied the spot).
        Effect::LeavePlay { target: Some(target), pokemon, dest, cause, how: crate::effects::LeaveHow::KnockOut | crate::effects::LeaveHow::Effect | crate::effects::LeaveHow::BenchShrink, .. } => {
            EventView { card: Some(pokemon).filter(|c| *c != crate::list::NO_CARD), ..crate::engine::knockout::leave_view(g, target, dest, cause) }
        }
        // Cards attached to a Pokémon, the Stadium: one view per card (`run::after_event`, `cards_zone::view_of`).
        Effect::LeavePlay { .. } => return None,
        Effect::ApplyEffect { target, cause, .. } => crate::engine::apply::apply_view(g, target, cause),
        Effect::TakePrizes { p, prizes, cause } => crate::engine::knockout::prizes_view(g, p as usize, prizes.len() as i32, cause),
        // The whole action (its pairs are in the effect); a trigger over one end would read `end` / `slot` per pair.
        Effect::MoveCounters { p, cause, .. } => EventView::new(EventKind::MoveCounters, cause, p, turn),
        // The turn-structure events (batch 8): the player whose turn begins or ends, the player being checked (nobody's
        // turn at Checkup, APR F).
        Effect::BeginTurn { p } => EventView::new(EventKind::BeginTurn, crate::cause::Cause::rule(crate::cause::RuleWhich::BeginTurn, p), p, turn),
        Effect::EndTurn { p } => EventView::new(EventKind::EndTurn, crate::cause::Cause::rule(crate::cause::RuleWhich::EndTurn, p), p, turn),
        Effect::BetweenTurns { p, .. } => EventView::new(EventKind::Checkup, crate::cause::Cause::rule(crate::cause::RuleWhich::Checkup, p), p, turn),
        _ => return None,
    })
}

/// Does the event of effect `e` fire the `Event::On` trigger `t` of card `me` (after the event is done)? The
/// declaring card must be in place for its origin and not blocked there (a Pokémon's Ability, the Stadium in
/// play, ...). Returns the program's player (the event's owner for a Stadium, else the card's owner) and the
/// event's spot as the picked slot.
pub(crate) fn fires_on(g: &mut Game, me: CardId, v: &super::event::EventView, t: &Trigger) -> crate::game::R<Option<(usize, u8)>> {
    let Event::On(pred) = &t.event else { return Ok(None) };
    // A card that just left play reacts to its own LeavePlay as it was in play: attached to the Pokémon in the event's
    // spot (Boomerang Energy, "if this card is discarded by an effect of an attack of the Pokémon it is attached to").
    let left = match (v.kind, v.card, v.slot) {
        (super::event::EventKind::LeavePlay, Some(c), Some(s)) if c == me && matches!(t.origin, RuleSource::Energy | RuleSource::Tool) => Some(super::passive::Located { owner: s.p as usize, held: Some(s) }),
        _ => None,
    };
    let Some(at) = super::passive::locate(g, me, t.origin).or(left) else { return Ok(None) };
    if !pred.eval(g, me, v)? {
        return Ok(None);
    }
    if super::passive::blocked(g, me, t.origin, at, v.slot) {
        return Ok(None);
    }
    let p = if t.origin == RuleSource::Stadium { v.owner as usize } else { g.st.owner(me) };
    let slot = v.slot.map_or(super::run::NONE, |s| s.p << 4 | s.s);
    Ok(Some((p, slot)))
}
