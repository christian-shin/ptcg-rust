//! Mega Kangaskhan ex (M1S, Basic Mega ex): Run Errand - once during your
//! turn, if this Pokémon is Active, draw 2 cards (1 Run Errand per turn).
//! Rapid-Fire Combo - 200+, flip until tails, 50 more per heads.
//!
//! Twinleaf keeps the once-per-turn flag on the player (`usedRunErrand`);
//! every copy in any zone resets it for the player whose turn ends
//! (`EndTurnEffect.player`), whoever owns the copy.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MegaKangaskhanex",
    mask: mask(&[k::END_TURN, k::POWER, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].used_run_errand = false;
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.active_pokemon(p) != Some(me) {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].used_run_errand {
            bail!("CANNOT_USE_POWER");
        }
        draw_cards(g, p, 2)?;
        ability_used(g, p, me);
        g.st.players[p].used_run_errand = true;
    }

    if was_attack_used(g, e, 0, me) {
        // FLIP_A_COIN_UNTIL_YOU_GET_TAILS_DO_X_MORE_DAMAGE_PER_HEADS(..., 50)
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = coin_flip_sequence(g, p, 0, CoinCb::SequenceCard { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let heads = (f.a[2] as u32).count_ones() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
        *damage += 50 * heads;
    }
    g.release_fx(atk);
    Ok(())
}
