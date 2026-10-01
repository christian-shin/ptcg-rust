//! Solrock (MEG, as Solrock ASC): Cosmo Beam — 70. If you don't have Lunatone
//! on your Bench, this attack does nothing. This attack's damage isn't
//! affected by Weakness or Resistance.
//!
//! Twinleaf (mega-evolution file): sets ignoreWeakness/ignoreResistance, then
//! looks for a Lunatone anywhere among your Pokémon in play (Active included,
//! not only the Bench); none → damage 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Solrock@ASC", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Effect::Attack { ignore_weakness, ignore_resistance, .. } = g.e_mut(e) {
            *ignore_resistance = true;
            *ignore_weakness = true;
        }
        let has = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).name == "Lunatone");
        if !has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 0;
            }
        }
    }
    Ok(())
}
