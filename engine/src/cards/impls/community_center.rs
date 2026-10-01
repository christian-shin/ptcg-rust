//! Community Center (TWM, stadium): once during each player's turn, if that
//! player has already played a Supporter from their hand, they may heal 10
//! damage from each of their Pokémon.
//!
//! Twinleaf: throws when `supporterTurn === 0`; otherwise a HealEffect(10)
//! on each of the player's Pokémon whose stadium effect isn't blocked
//! (undamaged Pokémon included).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CommunityCenter", mask: mask(&[k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].supporter_turn == 0 {
        bail!("CANNOT_USE_STADIUM");
    }
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let t = SlotRef::new(p, s);
        if is_stadium_effect_blocked(g, p, t, me) {
            continue;
        }
        g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 10 })?;
    }
    Ok(())
}
