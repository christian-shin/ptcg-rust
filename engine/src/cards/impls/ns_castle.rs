//! N's Castle (JTG, stadium): each N's Pokémon in play has no Retreat Cost.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsCastle", mask: mask(&[k::CHECK_RETREAT_COST, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => {
            let p = p as usize;
            let a = g.st.players[p].active;
            if is_stadium_effect_blocked(g, p, SlotRef::new(p, a), me) {
                return Ok(());
            }
            let ns = g.st.active_pokemon(p).map(|c| g.st.cdef(c).has_tag(tag::NS)).unwrap_or(false);
            if ns {
                if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
                    cost.clear();
                }
            }
            Ok(())
        }
        Effect::UseStadium { .. } => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
