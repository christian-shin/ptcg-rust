//! Backtrack Badge (PBL; Twinleaf "Retry Badge M5", class BacktrackBadge,
//! tool): once during your turn, after you flip any coins for an attack of
//! the [C] Pokémon this card is attached to, you may ignore the results of
//! those coin flips and begin flipping those coins again.
//!
//! Port of Twinleaf's `ATTACK_COIN_REFLIP_REDUCE_EFFECT` (source 'tool'),
//! which no earlier card used:
//! * EndTurnEffect removes COIN_REFLIP_AGAIN_USED from the ending player.
//! * A CoinFlipEffect (skipReflipTool false) during the attack phase, with the
//!   marker absent and `canOffer` true (attacking player's Active holds this
//!   tool, the tool is not blocked, the Active is a [C] Pokémon), is
//!   prevented and resolved here: coin, a wait prompt, then a Confirm
//!   (WANT_TO_USE_ABILITY). Declining runs the callback with the result;
//!   accepting adds the marker and flips again with both skip flags set.
//! * A CoinFlipSequenceEffect under the same conditions is prevented and run
//!   as a sequence of plain flips; when it completes the offer is made again
//!   (conditions re-checked): declining finishes the original callback, and
//!   accepting reruns the whole sequence with the skip flags set.
//! The coin session of the Badge is the one rule kept as code in this file (`Op::Custom`): the
//! frame carries its state. Events batch 4 (B4-OLD): the hook stays on the flip requests (`CoinFlipRequest`,
//! `CoinFlipSequence`), which ask for a flip before its result is used; every flip the Badge makes (its own and
//! the re-flips) is a CoinFlip event like any other (`cards` = callback and mode, `slot` = the player, `last` = the results, `iter[0]` =
//! the flip count of a finished sequence).
use crate::effects::{k, EffId, Effect};
use crate::game::{CoinCb, Game, R};
use crate::list::{CardId, CardList};
use crate::markers::COIN_REFLIP_AGAIN_USED;
use crate::prefabs::{confirmation_prompt, is_tool_blocked};
use crate::prompts::{PromptKind, Res};
use crate::spec::prelude::*;
use crate::spec::run::{Flow, Frame};
use crate::types::{ct, GamePhase};

/// No callback.
const NO_CALLBACK: u8 = 0xFF;

/// The Active Pokémon of the attacking player holds this unblocked Tool and is a [C] Pokémon.
fn can_offer(g: &mut Game, me: CardId, p: usize) -> bool {
    if g.st.phase != GamePhase::Attack || g.st.active_player as usize != p {
        return false;
    }
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return false;
    }
    if is_tool_blocked(g, p, me) {
        return false;
    }
    match g.st.slot_pokemon(p, a) {
        Some(c) => g.st.cdef(c).card_type.contains(&ct::COLORLESS),
        None => false,
    }
}

/// Does a coin flip (or a sequence of flips) of an attack fall under the Badge?
fn fires(g: &mut Game, me: CardId, e: EffId) -> Option<usize> {
    let (p, skip) = match *g.e(e) {
        Effect::CoinFlipRequest { p, skip_reflip_tool, .. } | Effect::CoinFlipSequence { p, skip_reflip_tool, .. } => (p as usize, skip_reflip_tool),
        _ => return None,
    };
    if skip || g.st.phase != GamePhase::Attack || g.st.players[p].marker.has(COIN_REFLIP_AGAIN_USED) || !can_offer(g, me, p) {
        return None;
    }
    Some(p)
}

fn exec(g: &mut Game, me: CardId, f: &mut Frame) -> R<Flow> {
    let e = f.eff;
    let p = f.p as usize;
    match *g.e(e) {
        // A single flip is prevented and resolved here: the coin, a wait prompt, then the offer.
        Effect::CoinFlipRequest { callback, cause, .. } => {
            g.set_prevent(e, true);
            let result = g.rng.coin();
            if let Effect::CoinFlipRequest { result: r, .. } = g.e_mut(e) {
                *r = Some(result);
            }
            crate::engine::condition::coin_flipped(g, p, crate::spec::event::CoinPurpose::Effect, result, cause)?;
            f.cards = [callback.unwrap_or(NO_CALLBACK), result as u8];
            f.slot = p as u8;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::Wait, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        // A sequence is prevented and run as a sequence of plain flips; the offer follows its end.
        Effect::CoinFlipSequence { mode, callback, cause, .. } => {
            g.set_prevent(e, true);
            f.cards = [callback, mode];
            f.slot = p as u8;
            g.coin_callbacks.push(CoinCb::SequenceCard { card: me, frame: f.frame_at(10) });
            let k2 = (g.coin_callbacks.len() - 1) as u8;
            g.run_fx_unit(Effect::CoinFlipSequence { p: p as u8, mode, callback: k2, skip_reflip_stadium: true, skip_reflip_tool: true, cause })?;
            Ok(Flow::Suspend)
        }
        _ => Ok(Flow::Next),
    }
}

fn resume(g: &mut Game, me: CardId, f: &mut Frame, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.sub {
        // Single flip: the animation wait is done, offer the reflip.
        1 => {
            confirmation_prompt(g, f.slot as usize, "WANT_TO_USE_ABILITY", f.cont(me, 2));
            Ok(Flow::Suspend)
        }
        2 => {
            let p = f.slot as usize;
            let callback = if f.cards[0] != NO_CALLBACK { Some(f.cards[0]) } else { None };
            let result = f.cards[1] != 0;
            if !first.as_bool() {
                let cb = match callback {
                    Some(k) => g.coin_callbacks.as_slice()[k as usize],
                    None => CoinCb::None,
                };
                g.run_coin_cb(cb, result)?;
                return Ok(Flow::Next);
            }
            g.st.players[p].marker.add_to_state(COIN_REFLIP_AGAIN_USED);
            g.run_fx_unit(Effect::CoinFlipRequest { p: p as u8, callback, result: None, skip_reflip_stadium: true, skip_reflip_tool: true, cause: f.cause })?;
            Ok(Flow::Next)
        }
        // Sequence finished (the core passes the results and the flip count).
        10 => {
            let p = f.slot as usize;
            f.last = first.as_int();
            f.iter[0] = results.get(1).map_or(0, |r| r.as_int()) as u8;
            if g.st.phase == GamePhase::Attack && !g.st.players[p].marker.has(COIN_REFLIP_AGAIN_USED) && can_offer(g, me, p) {
                confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", f.cont(me, 11));
                return Ok(Flow::Suspend);
            }
            finish(g, f)
        }
        11 => {
            let p = f.slot as usize;
            if !first.as_bool() {
                return finish(g, f);
            }
            g.st.players[p].marker.add_to_state(COIN_REFLIP_AGAIN_USED);
            g.run_fx_unit(Effect::CoinFlipSequence { p: p as u8, mode: f.cards[1], callback: f.cards[0], skip_reflip_stadium: true, skip_reflip_tool: true, cause: f.cause })?;
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

fn finish(g: &mut Game, f: &Frame) -> R<Flow> {
    let n = f.iter[0];
    let results = f.last as u32;
    let last = n > 0 && (results >> (n - 1)) & 1 == 1;
    g.finish_coin_sequence(f.cards[0], results, n, last)?;
    Ok(Flow::Next)
}

pub static SPEC: CardSpec = CardSpec {
    class: "BacktrackBadge",
    triggers: &[
        Trigger {
            origin: RuleSource::Tool,
            event: Event::Custom(CustomEventSpec { kinds: &[k::COIN_FLIP_REQUEST, k::COIN_FLIP_SEQUENCE], fires }),
            steps: &[Step::new(Op::Custom(CustomSpec { exec, resume }))],
        },
        // The once-per-turn use ends with the turn.
        Trigger {
            origin: RuleSource::Tool,
            event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
            steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "COIN_REFLIP_AGAIN_USED", from: MarkerFrom::Any }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
