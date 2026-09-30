//! Hero's Cape (TEF, ACE SPEC tool): the Pokémon this card is attached to
//! gets +100 HP.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HerosCape", mask: mask(&[k::CHECK_HP]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, target, card) = match *g.e(e) {
        Effect::CheckHp { p, target, card } => (p as usize, target, card),
        _ => return Ok(()),
    };
    if !g.st.slot(target.p as usize, target.s).tools.contains(me) {
        return Ok(());
    }
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    // `effect.hp += 100`: the setter writes hpBonus only when a Pokémon was captured.
    if card.is_some() {
        g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += 100;
    }
    Ok(())
}
