//! Umbreon ex (PRE, Tera): Moon Mirage — 160; your opponent's Active
//! Pokémon is now Confused. Onyx — discard all Energy from this Pokémon and
//! take a Prize card. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Onyx builds the energy map of `player.active`, reduces a
//! DiscardCardsEffect (target = the attacker's Active), then TAKE_X_PRIZES
//! (1): with 1 Prize left it is taken automatically, otherwise a
//! non-cancellable ChoosePrizePrompt (not secret).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Umbreonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let a = g.st.players[pu].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(pu, a), energy_map: SVec::new() })?;
        let mut cards = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for em in energy_map.iter() {
                cards.push(em.card);
            }
        }
        let a = g.st.players[pu].active;
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(pu, a) };
        g.run_fx(Effect::DiscardCards { b, cards })?;
        return take_one_prize(g, me, pu);
    }
    tera_rule(g, e, me);
    Ok(())
}

/// `TAKE_X_PRIZES(store, state, player, 1)`.
fn take_one_prize(g: &mut Game, me: CardId, p: usize) -> R {
    let left = g.st.players[p].prize_left();
    if left == 0 {
        return Ok(());
    }
    if left <= 1 {
        let pl = &g.st.players[p];
        let first = (0..pl.prize_count).find(|i| !pl.prizes[*i as usize].is_empty());
        if let Some(i) = first {
            crate::engine::check::take_specific_prizes(g, p, &[i], ListRef::Hand(p as u8), false)?;
        }
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize { count: 1, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: false, is_secret: false, destination: None },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    if let Some(Res::Prizes(ix)) = results.first() {
        crate::engine::check::take_specific_prizes(g, p, ix.as_slice(), ListRef::Hand(p as u8), false)?;
    }
    Ok(())
}
