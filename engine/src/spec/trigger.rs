//! Triggers (vocabulary v1 "Triggers"): an event, then steps. Each event
//! declares the effect kinds it reacts to (`kinds`) and recognizes its
//! effect in `fires`; the steps run as the card's trigger program.

use super::*;
use crate::effects::{mask, EffId, Effect, KindMask};
use crate::game::Game;
use crate::list::{CardId, CardList};

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
    /// "When you play this Pokémon from your hand onto your Bench" (any Play effect of this card).
    Play,
    /// A Pokémon is put onto an empty Bench spot (from the hand, deck or discard pile) during its
    /// owner's turn, whoever's it is; the program runs for that player with the Pokémon's slot
    /// as the picked slot (Risky Ruins).
    PutOnBench { basic: bool, not_type: Option<crate::types::CardType> },
}

pub struct OnEnterPlaySpec {
    pub method: EnterMethod,
}
pub struct OnMovedSpec {}
pub struct OnAttachSpec {}
pub struct OnKnockOutSpec {}
pub struct OnDamagedByAttackSpec {}
/// Between turns (Pokémon Checkup), once per turn change for the card's owner.
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
        Event::OnDiscarded(_) => mask(&[k::DISCARD_CARDS]),
        Event::OnCheckup(_) => mask(&[k::BETWEEN_TURNS]),
        Event::OnAfterAttackTriggers(_) => mask(&[k::AFTER_ATTACK_TRIGGERS]),
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
            Some(p)
        }
        Event::OnEnterPlay(w) => {
            let p = match (w.method, *g.e(e)) {
                (EnterMethod::Evolve, Effect::Evolve { p, card, .. }) if card == me => p as usize,
                (EnterMethod::Play, Effect::PlayPokemon { p, card, .. }) if card == me => p as usize,
                _ => return None,
            };
            if t.origin == RuleSource::Ability && crate::prefabs::is_ability_blocked(g, p, me, None) {
                return None;
            }
            Some(p)
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
            Some(pu)
        }
        Event::OnCheckup(_) => match *g.e(e) {
            Effect::BetweenTurns { .. } => Some(g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me))),
            _ => None,
        },
        Event::OnAfterAttackTriggers(_) => match *g.e(e) {
            Effect::AfterAttackTriggers { p, .. } => Some(p as usize),
            _ => None,
        },
        _ => unimplemented!("spec trigger not implemented yet (trigger.rs)"),
    }
}

/// The Pokémon slot an event is about (player << 4 | slot), or NONE: the slot register at the
/// start of the trigger's program.
pub(crate) fn event_slot(g: &Game, e: EffId, t: &Trigger) -> u8 {
    match (&t.event, *g.e(e)) {
        (
            Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::PutOnBench { .. } }),
            Effect::PlayPokemon { target, .. } | Effect::PlayPokemonFromDeck { target, .. } | Effect::PlayPokemonFromDiscard { target, .. },
        ) => target.p << 4 | target.s,
        _ => super::run::NONE,
    }
}
