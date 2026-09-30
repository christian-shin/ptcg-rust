//! Alakazam (M1S / MEG 56): Psychic Draw — when you play this Pokémon from
//! your hand to evolve, you may draw 3 cards. Hand Power — put 2 damage
//! counters on your opponent's Active Pokémon for each card in your hand.
use super::kadabra::{psychic_draw_reduce, psychic_draw_resume};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Alakazam@MEG", mask: mask(&[k::EVOLVE, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    psychic_draw_reduce(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            // PUT_X_DAMAGE_COUNTERS_ON_YOUR_OPPONENTS_ACTIVE_POKEMON(hand * 2).
            let x = g.st.players[p as usize].hand.len() as i32 * 2;
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::PutCounters { b, damage: 10 * x })?;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    psychic_draw_resume(g, f, results, 3)
}
