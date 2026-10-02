//! Jamming Tower (TWM / ASC, stadium): Pokémon Tools attached to each Pokémon
//! (both yours and your opponent's) have no effect.
//!
//! Twinleaf: throws CANNOT_USE_STADIUM on UseStadiumEffect. Every ToolEffect
//! finds the Pokémon holding the tool among the *effect player's own*
//! Pokémon (`effect.player.forEachPokemon(BOTTOM_PLAYER)`); when that target
//! exists and IS_STADIUM_EFFECT_BLOCKED (no stadium argument: NO_CARD here)
//! the Tool keeps working, otherwise the effect throws CANNOT_USE_POWER
//! (including when the tool sits on the other player's Pokémon).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "JammingTower", mask: mask(&[k::USE_STADIUM, k::TOOL]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::UseStadium { .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            bail!("CANNOT_USE_STADIUM");
        }
    }
    if let Effect::Tool { p, card } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            let p = p as usize;
            let mut target: Option<SlotRef> = None;
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.slot(p, s).tools.contains(card) {
                    target = Some(SlotRef::new(p, s));
                }
            }
            if let Some(t) = target {
                if is_stadium_effect_blocked(g, p, t, NO_CARD) {
                    return Ok(());
                }
            }
            bail!("CANNOT_USE_POWER");
        }
    }
    Ok(())
}
