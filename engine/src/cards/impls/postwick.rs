//! Postwick (JTG): the attacks of Hop's Pokémon (both players') do 30 more
//! damage to the opponent's Active Pokémon (before W/R).
//!
//! Twinleaf: DealDamageEffect only; block probe on the target's owner; the
//! target must be the attacker-opponent's Active; the source slot's Pokémon
//! needs the HOPS tag. Not limited to attacks with printed damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Postwick", mask: mask(&[k::DEAL_DAMAGE, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::DealDamage { b, .. } if g.st.stadium_card() == Some(me) => {
            let owner = b.target.p as usize;
            let opponent = 1 - b.player as usize;
            if is_stadium_effect_blocked(g, owner, b.target, me) {
                return Ok(());
            }
            if !(b.target.p as usize == opponent && b.target.s == g.st.players[opponent].active) {
                return Ok(());
            }
            let hops = g.st.slot_pokemon(b.source.p as usize, b.source.s).map(|c| g.st.cdef(c).has_tag(tag::HOPS)).unwrap_or(false);
            if !hops {
                return Ok(());
            }
            if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                *damage += 30;
            }
            Ok(())
        }
        Effect::UseStadium { .. } if g.st.stadium_card() == Some(me) => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
