//! Growing [G] Energy ("Grow [G] Energy M3", POR): provides [G]. The [G]
//! Pokémon this card is attached to gets +20 HP.
//!
//! Twinleaf: the [G] entry is pushed unconditionally (no EnergyEffect probe).
//! On CheckHp, if the target holds this card and the special energy isn't
//! blocked, a CheckPokemonType on the target decides: [G] → hp += 20 (the
//! setter writes hpBonus only when a Pokémon was captured).
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "GrowGrassEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::CHECK_HP]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckProvidedEnergy { source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            let mut provides = SVec::new();
            provides.push(ct::GRASS);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        Effect::CheckHp { p, target, card } => {
            if !g.st.slot(target.p as usize, target.s).cards.contains(me) {
                return Ok(());
            }
            if is_special_energy_blocked(g, p as usize, me, target, false) {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, target);
            let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
            if matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::GRASS)) && card.is_some() {
                g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += 20;
            }
        }
        _ => {}
    }
    Ok(())
}
