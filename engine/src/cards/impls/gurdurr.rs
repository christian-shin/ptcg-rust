//! Gurdurr (TWM 104): Knuckle Punch — 20. Superpower — 50; you may do 30
//! more damage. If you do, this Pokémon also does 30 damage to itself.
//!
//! Twinleaf: a non-yielding ConfirmPrompt (WANT_TO_USE_ABILITY); on yes,
//! `effect.damage += 30` and a PutDamageEffect(30) on `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gurdurr", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    f.a[0] = p as i32;
    confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let p = f.a[0] as usize;
    let r = if results.first().copied().unwrap_or(Res::Null).as_bool() {
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += 30;
        }
        let a = g.st.players[p].active;
        put_damage(g, atk, 30, SlotRef::new(p, a))
    } else {
        Ok(())
    };
    g.release_fx(atk);
    r
}
