//! Ethan's Pinsir (DRI 1): Vice Grip — 20. One-Point Return — 70+; 100 more
//! if any of your Ethan's Pokémon were Knocked Out by damage from an attack
//! during your opponent's last turn.
//!
//! Twinleaf: WAS_POKEMON_KNOCKED_OUT_DURING_OPPONENTS_LAST_TURN with
//! `{ byAttackDamage: true, tags: [ETHANS] }`: the by-attack flag, then any
//! KO entry (all KOs of that turn, not only attack KOs) carrying the tag.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansPinsir", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let pl = &g.st.players[p];
    let hit = pl.pokemon_knocked_out_by_attack_during_opponents_last_turn
        && pl.pokemon_knocked_out_last_turn_entries.iter().any(|d| crate::carddb::def(*d).has_tag(tag::ETHANS));
    if hit {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 100;
        }
    }
    Ok(())
}
