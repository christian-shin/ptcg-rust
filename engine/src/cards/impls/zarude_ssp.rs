//! Zarude (SSP): Leaf Drain — 20, heal 20 damage from this Pokémon.
//! Jungle Whip — 80+; you may put all Energy attached to this Pokémon into
//! your hand for 80 more damage.
//!
//! Twinleaf: HealTargetEffect(20) on the Active; Jungle Whip is a
//! ConfirmPrompt (WANT_TO_USE_ABILITY) whose yes-callback reads
//! CheckProvidedEnergyEffect on the Active, MOVE_CARDS those cards to the
//! hand, then adds 80 to the attack's damage.
use super::tynamo_sv11b::heal_own_active;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zarude@SSP", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        heal_own_active(g, e, 20)?;
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let p = f.a[0] as usize;
    let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
    let r = (|| -> R {
        if !yes {
            return Ok(());
        }
        let active = SlotRef::new(p, g.st.players[p].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: active, energy_map: SVec::new() })?;
        let mut cards: Vec<CardId> = Vec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for em in energy_map.iter() {
                cards.push(em.card);
            }
        }
        move_cards(g, active.list(), ListRef::Hand(p as u8), &cards, me)?;
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += 80;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
