//! Mega Latias ex (M1S / MEG 100): Strafe — 40; you may switch this Pokémon
//! with 1 of your Benched Pokémon. Illusory Impulse — 300; discard all Energy
//! from this Pokémon.
//!
//! Twinleaf: Strafe sets the instance field `strafeUsed` (canonical state),
//! and the next AfterAttackEffect of any attack consumes it (so it stays set
//! through the animation step). `player.bench.length > 0` is always true
//! (empty bench slots exist), so the Confirm prompt is created even with no
//! Benched Pokémon; SWITCH_ACTIVE_WITH_BENCHED then does nothing. Illusory
//! Impulse discards every card in the CheckProvidedEnergy map of the Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaLatiasex", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        g.st.cards[me as usize].strafe_used = true;
    }

    if let Effect::AfterAttack { p, .. } = *g.e(e) {
        if g.st.cards[me as usize].strafe_used {
            g.st.cards[me as usize].strafe_used = false;
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p as usize, "WANT_TO_SWITCH_POKEMON", Cont::Card { card: me, frame: f });
        }
    }

    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let pp = p as usize;
            let a = g.st.players[pp].active;
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(pp, a), energy_map: SVec::new() })?;
            let mut cards = SVec::new();
            if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                for m in energy_map.iter() {
                    cards.push(m.card);
                }
            }
            let target = SlotRef::new(pp, a);
            g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, cards })?;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let first = results.first().copied().unwrap_or(Res::Null);
    if first.as_bool() {
        switch_active_with_benched(g, f.a[0] as usize);
    }
    Ok(())
}
