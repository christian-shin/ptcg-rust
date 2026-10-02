//! Voltaic [L] Energy ("Bolty [L] Energy M5", PBL): provides [L]. Attacks used
//! by the [L] Pokémon this card is attached to do 20 more damage to your
//! opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: the [L] entry is pushed unless an EnergyEffect probe throws. On a
//! DealDamageEffect whose source slot holds this card (unless the special
//! energy is blocked): a CheckPokemonType on the source must contain [L];
//! then `damage > 0 && target === opponent.active` → damage += 20.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "VoltaicLightningEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::DEAL_DAMAGE]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Energy { p, card: me }).is_err() {
                return Ok(());
            }
            let mut provides = SVec::new();
            provides.push(ct::LIGHTNING);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        Effect::DealDamage { b, damage } => {
            if !g.st.slot(b.source.p as usize, b.source.s).cards.contains(me) {
                return Ok(());
            }
            if is_special_energy_blocked(g, b.player as usize, me, b.source, false) {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, b.source);
            let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target: b.source, card_types: types })?;
            if !matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::LIGHTNING)) {
                return Ok(());
            }
            let o = b.opponent as usize;
            if damage > 0 && b.target.p as usize == o && b.target.s == g.st.players[o].active {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 20;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
