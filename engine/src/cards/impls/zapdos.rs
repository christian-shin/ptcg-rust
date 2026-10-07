//! Zapdos (TWM): Thunder Wave — flip a coin, if heads the opponent's Active
//! Pokémon is now Paralyzed. Thunderbolt — 190; discard all Energy from this
//! Pokémon.
//!
//! Twinleaf: the coin callback reduces an AddSpecialConditionsEffect; the
//! discard is one DiscardCardsEffect with every card of the Active's
//! CheckProvidedEnergy map, aimed at `player.active`.
//!
//! Fixed (phase 4b, R2): Thunderbolt discarded the Energy in the attack
//! handler, before the damage (Voltaic Lightning Energy's +20 was lost); it
//! now discards in AfterAttackEffect with a fresh AttackEffect's data.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zapdos", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(0);
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
        return Ok(());
    }
    if after_attack_used(g, e, 1, me) {
        discard_all_energy_from_active(g, e)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    let r = if heads { add_special_conditions_to_opponent_active(g, atk, &[SpecialCondition::Paralyzed]) } else { Ok(()) };
    g.release_fx(atk);
    r
}

/// `CheckProvidedEnergyEffect(player)` on the Active, then one
/// DiscardCardsEffect of every mapped card aimed at `player.active`.
/// Returns how many cards the effect listed. `e` is an Attack effect, or the
/// AfterAttack effect (the `new AttackEffect(player, opponent, attack)` of the
/// phase 4b fixes: its source is the player's Active).
pub fn discard_all_energy_from_active(g: &mut Game, e: EffId) -> R<usize> {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        Effect::AfterAttack { p, opp, attack } => (p, opp, attack, SlotRef::new(p as usize, g.st.players[p as usize].active)),
        _ => return Ok(0),
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
    let n = cards.len();
    let target = SlotRef::new(pu, g.st.players[pu].active);
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
    g.run_fx(Effect::DiscardCards { b, cards })?;
    Ok(n)
}
