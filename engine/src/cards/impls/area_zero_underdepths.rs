//! Area Zero Underdepths (SCR, stadium): each player with any Tera Pokémon in
//! play can have up to 8 Benched Pokémon; otherwise the Bench shrinks back to
//! 5 (handled by the check-state bench size change).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "AreaZeroUnderdepths", mask: mask(&[k::CHECK_TABLE_STATE, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn is_tera(g: &Game, p: usize, s: SlotId) -> bool {
    g.st.slot_pokemon(p, s).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_TERA)).unwrap_or(false)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    match *g.e(e) {
        Effect::CheckTableState { .. } => {
            let mut sizes = [5u8; 2];
            for (p, size) in sizes.iter_mut().enumerate() {
                let pl = &g.st.players[p];
                let mut tera = 0;
                if is_tera(g, p, pl.active) {
                    tera += 1;
                }
                for &b in pl.bench.iter() {
                    if is_tera(g, p, b) {
                        tera += 1;
                    }
                }
                *size = if tera >= 1 { 8 } else { 5 };
            }
            if let Effect::CheckTableState { bench_sizes } = g.e_mut(e) {
                *bench_sizes = sizes;
            }
            Ok(())
        }
        Effect::UseStadium { .. } => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
