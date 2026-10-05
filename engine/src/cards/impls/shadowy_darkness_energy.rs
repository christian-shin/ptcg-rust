//! Shadowy Darkness Energy ("Shadow Darkness Energy M5", PBL): provides [D].
//! Prevent all damage done by your opponent's attacks to the Benched [D]
//! Pokémon this card is attached to.
//!
//! Twinleaf: the [D] entry is pushed unless an EnergyEffect probe throws.
//! A DealDamage / PutDamage effect during the ATTACK phase on a benched slot
//! holding this card, from the slot owner's opponent, gets `damage = 0`
//! unless the special energy is blocked or a CheckPokemonTypeEffect on the
//! slot lacks [D].
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "ShadowyDarknessEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::DEAL_DAMAGE, k::PUT_DAMAGE]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Energy { p, card: me }).is_err() {
                return Ok(());
            }
            let mut provides = SVec::new();
            provides.push(ct::DARK);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
            return Ok(());
        }
        Effect::DealDamage { b, .. } | Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    if ignores_defender_effects(g, &b) {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    let t = b.target;
    let owner = t.p as usize;
    if !g.st.slot(owner, t.s).cards.contains(me) {
        return Ok(());
    }
    if t.s == g.st.players[owner].active {
        return Ok(());
    }
    if 1 - owner != b.player as usize {
        return Ok(());
    }
    if is_special_energy_blocked(g, owner, me, t, false) {
        return Ok(());
    }
    let types = crate::engine::game_effect::pokemon_types(g, t);
    let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
    if !matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::DARK)) {
        return Ok(());
    }
    match g.e_mut(e) {
        Effect::DealDamage { damage, .. } | Effect::PutDamage { damage, .. } => *damage = 0,
        _ => {}
    }
    Ok(())
}
