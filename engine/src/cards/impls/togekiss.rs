//! Togekiss (SSP / ASC): Wonder Kiss - whenever your opponent's Active Pokémon
//! gets Knocked Out, flip a coin; if heads, take 1 more Prize card for that
//! Knock Out (does not stack). Speed Wing - 140.
//!
//! Twinleaf: on a KnockOutEffect for the owner's Active, with this card in
//! play on the other side, unless the Ability is blocked and the sourceless
//! marker TOGEKISS_KNOCKOUT_FLIP isn't set: set the marker, flip (the
//! KnockOutEffect is retained across the flip), `prizeCount += 1` on heads
//! when it is > 0, then remove the marker.
//!
//! Fixed (phase 4b, R7F-1; rulings 1591, 1619, 1623): the handler used to
//! require the ATTACK phase of Togekiss' owner, so a Knock Out by Poison or
//! Burn in Pokémon Checkup, or by an Ability (Cursed Blast), took no extra
//! Prize. Any Knock Out of the opponent's Active counts.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Togekiss@SSP|ASC", mask: mask(&[k::KNOCK_OUT]), reduce, resume: None, coin: Some(coin), can_play: None };

fn flip_marker() -> crate::markers::MarkerName {
    crate::marker!("TOGEKISS_KNOCKOUT_FLIP")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (owner, target) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    if target.s != g.st.players[owner].active {
        return Ok(());
    }
    let attacker = 1 - owner;
    let in_play = {
        let pl = &g.st.players[attacker];
        pl.slots[pl.active as usize].cards.contains(me) || pl.bench.iter().any(|b| pl.slots[*b as usize].cards.contains(me))
    };
    if !in_play {
        return Ok(());
    }
    if is_ability_blocked(g, attacker, me, None) {
        return Ok(());
    }
    if g.st.players[owner].marker.has(flip_marker()) {
        return Ok(());
    }
    g.st.players[owner].marker.add_to_state(flip_marker());
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    f.a[0] = owner as i32;
    if let Err(err) = g.coin_flip(attacker, CoinCb::Card { card: me, frame: f }) {
        g.release_fx(e);
        return Err(err);
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let e = f.e[0];
    let owner = f.a[0] as usize;
    if heads {
        if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
            if *prize_count > 0 {
                *prize_count += 1;
            }
        }
    }
    g.st.players[owner].marker.remove(flip_marker());
    g.release_fx(e);
    Ok(())
}
