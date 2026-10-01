//! Palafin (TEF): Vanguard Punch — 130; this Pokémon also does 10 damage to
//! itself for each damage counter on it (DealDamageEffect of `active.damage`
//! on the Active, computed before the main damage). Double Hit — 90x; flip 2
//! coins (`effect.damage = 90 * heads`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Palafin@Palafin TEF", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let b = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => {
                let a = g.st.players[p as usize].active;
                let target = SlotRef::new(p as usize, a);
                AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
            }
            _ => return Ok(()),
        };
        let self_damage = g.st.slot(b.target.p as usize, b.target.s).damage;
        g.run_fx(Effect::DealDamage { b, damage: self_damage })?;
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = coin_flip_sequence(g, p, 2, CoinCb::SequenceCard { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage == 1 {
        let atk = f.e[0];
        let heads = (f.a[2] as u32).count_ones() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 90 * heads;
        }
        g.release_fx(atk);
    }
    Ok(())
}
