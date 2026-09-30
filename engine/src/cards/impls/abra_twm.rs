//! Abra (TWM): Teleporter — once during your turn, if this Pokémon is in the
//! Active Spot, shuffle it and all attached cards into your deck. Beam — 10.
//!
//! Twinleaf: no once-per-turn marker (the card leaves play); the final
//! ShuffleDeckPrompt has no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Abra@TWM", mask: mask(&[k::POWER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_power_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Power { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.active_pokemon(p) != Some(me) {
        bail!("CANNOT_USE_POWER");
    }
    let a = g.st.players[p].active;
    move_pokemon_off_board(g, SlotRef::new(p, a), ListRef::Deck(p as u8), me)?;
    let a = g.st.players[p].active;
    crate::engine::game_effect::clear_effects(&mut g.st.players[p].slots[a as usize]);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    if let Some(Res::Order(o)) = results.first() {
        crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
    }
    Ok(())
}
