//! Boss's Orders (PAL): switch 1 of the opponent's Benched Pokémon with
//! their Active Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BossOrders", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        // Can't be played when it would not change the game state (Advanced Rulebook B-03, ruling 851);
        // used through an attack (Look-Alike Show) it just does nothing.
        if !trainer_via_attack(g, e) {
            let o = &g.st.players[1 - p];
            if !o.bench.iter().any(|b| !o.slots[*b as usize].cards.is_empty()) {
                bail!("CANNOT_PLAY_THIS_CARD");
            }
        }
        switch_in_opponent_benched_pokemon(g, p, false);
    }
    Ok(())
}
