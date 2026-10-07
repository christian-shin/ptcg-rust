//! Pawmot (PFL 34): Voltaic Fist — 130; you may have this Pokémon also do 60
//! damage to itself and make your opponent's Active Pokémon Paralyzed.
//!
//! Twinleaf: CONFIRMATION_PROMPT (WANT_TO_USE_ABILITY); on yes a DealDamageEffect
//! aimed at the attacking slot, then an AddSpecialConditionsEffect [PARALYZED]
//! on the opponent's Active.
use super::tapu_bulu::this_pokemon_does_damage_to_itself;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PawmotPFLPool", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
    let r = (|| -> R {
        if yes {
            this_pokemon_does_damage_to_itself(g, atk, 60)?;
            add_special_conditions_to_opponent_active(g, atk, &[SpecialCondition::Paralyzed])?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
