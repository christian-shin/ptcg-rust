//! Yveltal (SFA 35): Corrosive Winds — put 2 damage counters on each of your
//! opponent's Pokémon that has any damage counters. Destructive Beam — 100;
//! flip a coin, if heads discard an Energy from the opponent's Active.
//!
//! Twinleaf: no coin flip when the Active has no Energy card; the discard is
//! a ChooseCardsPrompt on the Active then a DiscardCardsEffect.
use crate::cards::prelude::*;
use super::trubbish::{discard_an_energy_from_opponents_active, discard_chosen};

pub static IMPL: CardImpl = CardImpl { class: "Yveltal@SFA", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let o = opp as usize;
            for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
                if g.st.slot(o, s).damage > 0 {
                    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, s) };
                    g.run_fx(Effect::PutCounters { b, damage: 20 })?;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        if !g.st.slot(o, a).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    if !heads {
        g.release_fx(atk);
        return Ok(());
    }
    discard_an_energy_from_opponents_active(g, me, atk, 2)
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 2 {
        return discard_chosen(g, f, results);
    }
    Ok(())
}
