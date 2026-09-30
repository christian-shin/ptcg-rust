//! Hisuian Arcanine (TWM): Proud Fangs — 30+; 90 more if your Benched
//! Pokémon have any damage counters. Searing Flame — 90, the opponent's
//! Active Pokémon is now Burned.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HisuianArcanine", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let active = g.st.players[p].active;
            let damaged = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(s, _, _)| *s != active && g.st.slot(p, *s).damage > 0);
            if damaged {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 90;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Burned])?;
    }
    Ok(())
}
