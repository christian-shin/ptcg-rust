//! Fennel (SV11B): heal 40 damage from each of your Pokémon.
//!
//! Twinleaf: moves to the supporter pile with preventDefault, then one
//! HealEffect per Pokémon (Active first, then the Bench in order).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Fennel", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, s), damage: 40 })?;
    }
    Ok(())
}
