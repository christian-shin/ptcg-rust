//! Cynthia's Garchomp ex (DRI): Corkscrew Dive - 100; you may draw cards until
//! you have 6 cards in your hand. Draconic Buster - 260; discard all Energy
//! from this Pokémon.
//!
//! Twinleaf: Corkscrew Dive asks (WANT_TO_DRAW_UNTIL_6) only when the hand has
//! fewer than 6 cards and the deck isn't empty; on yes the hand is refilled
//! with a plain `deck.moveTo(hand, n)` (no MoveCardsEffect), `n` computed
//! when the prompt resolves. Draconic Buster discards every card of the
//! Active's CheckProvidedEnergyEffect in one DiscardCardsEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "CynthiasGarchompex",
    mask: mask(&[k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].hand.len() >= 6 || g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        confirmation_prompt(g, p, "WANT_TO_DRAW_UNTIL_6", Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let active = SlotRef::new(pu, g.st.players[pu].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 64> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                cards.push(m.card);
            }
        }
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: active };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
    if yes {
        let n = 6 - g.st.players[p].hand.len() as i32;
        if n > 0 {
            g.move_to(ListRef::Deck(p as u8), ListRef::Hand(p as u8), Some(n as usize));
        }
    }
    Ok(())
}
