//! Hop's Trevenant (ASC 96): Horrifying Revenge — 30+; 100 more if any of
//! your Hop's Pokémon were Knocked Out by damage from an attack during your
//! opponent's last turn. Corner — 90; during your opponent's next turn the
//! Defending Pokémon can't retreat.
//!
//! Twinleaf: WAS_POKEMON_KNOCKED_OUT_DURING_OPPONENTS_LAST_TURN with
//! `{ byAttackDamage: true, tags: [HOPS] }`, then BLOCK_RETREAT.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsTrevenantASCPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let hit = pl.pokemon_knocked_out_by_attack_during_opponents_last_turn
            && pl.pokemon_knocked_out_last_turn_entries.iter().any(|d| crate::carddb::def(*d).has_tag(tag::HOPS));
        if hit {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 100;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        block_retreat(g, e)?;
    }
    Ok(())
}
