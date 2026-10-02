//! Punk Helmet (PFL, tool): if the [D] Pokémon this card is attached to is in
//! the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if this Pokémon is Knocked Out), place 4 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: on AfterDamageEffect against the holder: needs damage above 0,
//! an opposing attacker and the holder in the Active Spot; then the tool
//! probe (a bare ToolEffect for the attacking player) and a
//! CheckPokemonTypeEffect on the holder; with Dark type during the attack
//! phase `source.damage += 40` directly (no damage effect, no KO check).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PunkHelmet", mask: mask(&[k::AFTER_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (b, damage) = match *g.e(e) {
        Effect::AfterDamage { b, damage } => (b, damage),
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    if damage <= 0 || b.player == t.p || g.st.players[t.p as usize].active != t.s {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: b.player, card: me }).is_err() {
        return Ok(());
    }
    let types = crate::engine::game_effect::pokemon_types(g, t);
    let (ce, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
    let dark = matches!(ce, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::DARK));
    if dark && g.st.phase == GamePhase::Attack {
        let s = b.source;
        g.st.players[s.p as usize].slots[s.s as usize].damage += 40;
    }
    Ok(())
}
