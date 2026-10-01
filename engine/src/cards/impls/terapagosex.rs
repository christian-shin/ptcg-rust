//! Terapagos ex (SCR, Tera): Unified Beatdown — 30 for each of your Benched
//! Pokémon; if you go second, you can't use it during your first turn.
//! Crown Opal — 180; during your opponent's next turn, prevent all damage
//! done to this Pokémon by attacks from Basic non-[C] Pokémon.
//!
//! Twinleaf: Unified Beatdown throws CANNOT_USE_ATTACK whenever
//! `state.turn <= 2` (either player). Crown Opal arms PREVENT_DAMAGE with
//! `{ sourceStage: BASIC, sourceCardTypes: [every type but C] }`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Terapagosex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let bench = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32;
        if g.st.turn <= 2 {
            bail!("CANNOT_USE_ATTACK");
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = bench * 30;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let mut types = SVec::new();
        for t in [ct::GRASS, ct::FIRE, ct::WATER, ct::LIGHTNING, ct::PSYCHIC, ct::FIGHTING, ct::DARK, ct::METAL, ct::FAIRY, ct::DRAGON] {
            types.push(t);
        }
        let filter = PreventFilter { source_stage: Some(Stage::Basic as u8), source_card_types: Some(types), source_has_ability: false };
        prevent_damage_filtered(g, e, filter)?;
    }
    tera_rule(g, e, me);
    Ok(())
}
