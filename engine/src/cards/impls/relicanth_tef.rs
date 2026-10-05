//! Relicanth (TEF): Memory Dive — each of your evolved Pokémon can use any
//! attack from its previous Evolutions. Razor Fin — 30.
//!
//! The Twinleaf class is `set-temporal-forces/relicanth.ts` (the one the TEF
//! set index registers; `data/pool.json` points at an unused TWM file).
//! * CheckTableStateEffect: `effect.player` is never set, so
//!   `owner !== player` always returns (after findCardList, which throws
//!   INVALID_GAME_STATE when this card is in no list).
//! * CheckPokemonAttacksEffect: when this card is in play for the effect's
//!   player and its Ability is not locked (phase 4b: the lock probe is new),
//!   an evolved Active Pokémon adds the attacks of every other Pokémon card in
//!   its slot. Phase 4b: it used to add them for every evolved in-play Pokémon
//!   (the Active could use the previous-Evolution attacks of Benched Pokémon,
//!   and a Basic Active those of a Benched evolved one). Each Relicanth in
//!   play still adds them again.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Relicanth@TEF",
    mask: mask(&[k::CHECK_TABLE_STATE, k::CHECK_POKEMON_ATTACKS]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckTableState { .. } => {
            if g.st.locate(me).is_none() {
                bail!("INVALID_GAME_STATE");
            }
        }
        Effect::CheckPokemonAttacks { p, .. } => {
            let owner = match g.st.locate(me) {
                None => bail!("INVALID_GAME_STATE"),
                Some(l) => l.owner(),
            };
            let p = p as usize;
            if owner != Some(p) {
                return Ok(());
            }
            let in_play = for_each_pokemon(g, p, PlayerType::BottomPlayer);
            if !in_play.iter().any(|(_, c, _)| *c == me) {
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let mut add: SVec<AttackRef, 32> = SVec::new();
            let active = g.st.players[p].active;
            if let Some(top) = g.st.slot_pokemon(p, active) {
                if g.st.cdef(top).stage != Stage::Basic as u8 {
                    for c in g.st.slot(p, active).cards.iter() {
                        let d = g.st.cdef(c);
                        if d.is_pokemon() && c != top {
                            for i in 0..d.attacks.len() {
                                add.push(AttackRef { card: c, index: i as u8 });
                            }
                        }
                    }
                }
            }
            if let Effect::CheckPokemonAttacks { attacks, .. } = g.e_mut(e) {
                for a in add.iter() {
                    attacks.push(*a);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
