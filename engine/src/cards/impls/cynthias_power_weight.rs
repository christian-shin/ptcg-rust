//! Cynthia's Power Weight (DRI, tool): the Cynthia's Pokémon this card is
//! attached to gets +70 HP.
//!
//! Twinleaf: the tool block probe is a bare ToolEffect (no
//! stadium-and-tool-no-effect check).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CynthiasPowerWeight", mask: mask(&[k::CHECK_HP]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, target, captured) = match *g.e(e) {
        Effect::CheckHp { p, target, card } => (p, target, card),
        _ => return Ok(()),
    };
    if !g.st.slot(target.p as usize, target.s).tools.contains(me) {
        return Ok(());
    }
    let card = g.st.slot_pokemon(target.p as usize, target.s);
    if g.run_fx(Effect::Tool { p, card: me }).is_err() {
        return Ok(());
    }
    let card = match card {
        Some(c) => c,
        None => return Ok(()),
    };
    // `effect.hp += 70`: the setter writes hpBonus only when a Pokémon was captured.
    if g.st.cdef(card).has_tag(tag::CYNTHIAS) && captured.is_some() {
        g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += 70;
    }
    Ok(())
}
