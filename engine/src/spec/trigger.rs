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
    /// A rule no event expresses (Backtrack Badge): the card names the effect kinds it reacts to and
    /// decides in its own function whether it fires (returning the program's player).
    Custom(CustomEventSpec),
}

pub struct CustomEventSpec {
    pub kinds: &'static [u32],
    pub fires: fn(&mut Game, CardId, EffId) -> Option<usize>,
}

/// How a Pokémon came into play.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterMethod {
    /// "When you play this Pokémon from your hand to evolve" (any Evolve effect of this card,
    /// Rare Candy included).
    Evolve,
    /// "When you play this Pokémon from your hand onto your Bench" (any Play effect of this card).
    Play,
    /// A Pokémon is put onto an empty Bench spot (from the hand, deck or discard pile) during its
    /// owner's turn, whoever's it is; the program runs for that player with the Pokémon's slot
    /// as the picked slot (Risky Ruins).
    PutOnBench { basic: bool, not_type: Option<crate::types::CardType> },
}

/// This card enters play.
pub struct OnEnterPlaySpec {
    pub method: EnterMethod,
}
/// This Pokémon moved from the Active Spot to the Bench during its owner's turn.
pub struct OnMovedSpec {}
/// This Energy card is attached to a Pokémon, from any zone.
pub struct OnAttachSpec {}
/// This Pokémon, in the Active Spot, is Knocked Out by damage from an attack of the opponent's
/// Pokémon. The Attacking Pokémon's slot is the program's picked slot (`SlotExpr::Picked`).
pub struct OnKnockOutSpec {}
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
/// Pokémon Checkup (between turns), for every copy of the card in any zone; the program's player is the
/// player being checked.
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
        Event::OnEnterPlay(w) => match w.method {
            EnterMethod::Evolve => mask(&[k::EVOLVE]),
            EnterMethod::Play => mask(&[k::PLAY_POKEMON]),
            EnterMethod::PutOnBench { .. } => mask(&[k::PLAY_POKEMON, k::PLAY_POKEMON_FROM_DECK, k::PLAY_POKEMON_FROM_DISCARD]),
        },
        Event::OnKnockOut(_) => mask(&[k::KNOCK_OUT]),
        Event::OnAttach(_) => mask(&[k::ATTACH_ENERGY]),
        Event::OnMoved(_) => mask(&[k::MOVED_FROM_ACTIVE_TO_BENCH]),
        Event::OnDamagedByAttack(_) => mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]),
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
        Event::OnEnterPlay(OnEnterPlaySpec { method: m @ (EnterMethod::Play | EnterMethod::Evolve) }) => {
            let p = match (m, *g.e(e)) {
                (EnterMethod::Play, Effect::PlayPokemon { p, card, .. }) | (EnterMethod::Evolve, Effect::Evolve { p, card, .. }) if card == me => p as usize,
                _ => return None,
            };
            if super::passive::blocked(g, me, t.origin, super::passive::Located { owner: p, held: None }, None) {
                return None;
            }
            Some((p, None))
        }
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
        Event::OnDamagedByAttack(d) => {
            // Where the card is: part of the Pokémon (an Ability) or attached to it (a Tool).
            let holds = |g: &Game, target: crate::effects::SlotRef| match t.origin {
                RuleSource::Tool => g.st.slot(target.p as usize, target.s).tools.contains(me),
                _ => g.st.slot(target.p as usize, target.s).cards.contains(me),
            };
            match *g.e(e) {
                Effect::AfterDamage { b, damage } => {
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
        Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::PutOnBench { basic, not_type } }) => {
            let (p, card, target) = match *g.e(e) {
                Effect::PlayPokemon { p, card, target, .. } | Effect::PlayPokemonFromDeck { p, card, target } | Effect::PlayPokemonFromDiscard { p, card, target } => {
                    (p as usize, card, target)
                }
                _ => return None,
            };
            let tp = target.p as usize;
            if !g.st.slot(tp, target.s).cards.is_empty() || !g.st.players[p].bench.contains(&target.s) || g.st.active_player as usize != p {
                return None;
            }
            let d = g.st.cdef(card);
            if (*basic && d.stage != crate::types::Stage::Basic as u8) || not_type.map_or(false, |t| d.card_type.contains(&t)) {
                return None;
            }
            if t.origin == RuleSource::Stadium && (g.st.stadium_card() != Some(me) || crate::prefabs::is_stadium_effect_blocked(g, p, target, me)) {
                return None;
            }
            Some((p, Some(target.p << 4 | target.s)))
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
            Some((pu, None))
        }
        Event::Custom(c) => (c.fires)(g, me, e).map(|p| (p, None)),
        Event::OnCheckup(_) => match *g.e(e) {
            Effect::BetweenTurns { p, .. } if g.st.phase == crate::types::GamePhase::BetweenTurns => Some((p as usize, None)),
            _ => None,
        },
        Event::OnAfterAttackTriggers(_) => match *g.e(e) {
            Effect::AfterAttackTriggers { p, .. } => Some((p as usize, None)),
            _ => None,
        },
    }
}
