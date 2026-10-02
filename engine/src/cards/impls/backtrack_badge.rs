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
use crate::cards::prelude::*;
use crate::markers::COIN_REFLIP_AGAIN_USED;

pub static IMPL: CardImpl = CardImpl {
    class: "BacktrackBadge",
    mask: mask(&[k::END_TURN, k::COIN_FLIP, k::COIN_FLIP_SEQUENCE]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

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

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::EndTurn { p } => {
            g.st.players[p as usize].marker.remove(COIN_REFLIP_AGAIN_USED);
            Ok(())
        }
        Effect::CoinFlip { p, callback, skip_reflip_tool, .. } => {
            let p = p as usize;
            if skip_reflip_tool || g.st.phase != GamePhase::Attack || g.st.players[p].marker.has(COIN_REFLIP_AGAIN_USED) || !can_offer(g, me, p) {
                return Ok(());
            }
            g.set_prevent(e, true);
            let result = g.rng.coin();
            if let Effect::CoinFlip { result: r, .. } = g.e_mut(e) {
                *r = Some(result);
            }
            let mut f = CardFrame::at(1);
            f.a[0] = callback.map(|k| k as i32).unwrap_or(-1);
            f.a[2] = result as i32;
            f.l[0] = p as u8;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::Wait, Cont::Card { card: me, frame: f });
            Ok(())
        }
        Effect::CoinFlipSequence { p, mode, callback, skip_reflip_tool, .. } => {
            let p = p as usize;
            if skip_reflip_tool || g.st.phase != GamePhase::Attack || g.st.players[p].marker.has(COIN_REFLIP_AGAIN_USED) || !can_offer(g, me, p) {
                return Ok(());
            }
            g.set_prevent(e, true);
            let mut f = CardFrame::at(10);
            f.a[0] = callback as i32;
            f.a[1] = mode as i32;
            f.l[0] = p as u8;
            g.coin_callbacks.push(CoinCb::SequenceCard { card: me, frame: f });
            let k2 = (g.coin_callbacks.len() - 1) as u8;
            g.run_fx(Effect::CoinFlipSequence { p: p as u8, mode, callback: k2, skip_reflip_stadium: true, skip_reflip_tool: true })?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.l[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        // Single flip: the animation wait is done, offer the reflip.
        1 => {
            let mut nf = f;
            nf.stage = 2;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let callback = if f.a[0] >= 0 { Some(f.a[0] as u8) } else { None };
            let result = f.a[2] != 0;
            if !first.as_bool() {
                let cb = match callback {
                    Some(k) => g.coin_callbacks.as_slice()[k as usize],
                    None => CoinCb::None,
                };
                return g.run_coin_cb(cb, result);
            }
            g.st.players[p].marker.add_to_state(COIN_REFLIP_AGAIN_USED);
            g.run_fx(Effect::CoinFlip { p: p as u8, callback, result: None, skip_reflip_stadium: true, skip_reflip_tool: true })?;
            Ok(())
        }
        // Sequence finished (core fills a[2] = results, a[3] = flips).
        10 => {
            if g.st.phase == GamePhase::Attack && !g.st.players[p].marker.has(COIN_REFLIP_AGAIN_USED) && can_offer(g, me, p) {
                let mut nf = f;
                nf.stage = 11;
                confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            finish(g, f)
        }
        11 => {
            if !first.as_bool() {
                return finish(g, f);
            }
            g.st.players[p].marker.add_to_state(COIN_REFLIP_AGAIN_USED);
            g.run_fx(Effect::CoinFlipSequence { p: p as u8, mode: f.a[1] as u8, callback: f.a[0] as u8, skip_reflip_stadium: true, skip_reflip_tool: true })?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn finish(g: &mut Game, f: CardFrame) -> R {
    let n = f.a[3] as u8;
    let results = f.a[2] as u32;
    let last = n > 0 && (results >> (n - 1)) & 1 == 1;
    g.finish_coin_sequence(f.a[0] as u8, results, n, last)
}
