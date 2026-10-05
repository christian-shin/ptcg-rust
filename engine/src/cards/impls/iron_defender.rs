//! Iron Defender (MEG): during your opponent's next turn, all of your [M]
//! Pokémon take 30 less damage from attacks from your opponent's Pokémon.
//!
//! Twinleaf: every Iron Defender instance (in any zone) runs a
//! CheckPokemonTypeEffect on the target of every PutDamageEffect; if the
//! owner of the zone holding that instance has the marker sourced by it and
//! the target is an [M] Pokémon, it takes 30 less, but only in the attack
//! phase, when the target is that owner's Pokémon and the attacker is not
//! (phase 4b: it used to reduce any [M] target, either side, any attacker).
//! Each played copy has its own marker, so reductions stack. The markers (all
//! sources) are removed from the opponent of whoever ends a turn.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "IronDefender",
    mask: mask(&[k::TRAINER, k::PUT_DAMAGE, k::END_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn iron() -> crate::markers::MarkerName {
    marker!("IRON_DEFENDER_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        g.st.players[p].marker.add(iron(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }

    if let Effect::PutDamage { b, .. } = *g.e(e) {
        if ignores_defender_effects(g, &b) {
            return Ok(());
        }
        let owner = match g.st.locate(me).and_then(|l| l.owner()) {
            Some(o) => o,
            None => bail!("INVALID_GAME_STATE"),
        };
        let has = g.st.players[owner].marker.has_from(iron(), me);
        let target = b.target;
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        let metal = matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::METAL));
        if has && metal && g.st.phase == GamePhase::Attack && b.target.p as usize == owner && b.source.p as usize != owner {
            if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
                *damage -= 30;
            }
        }
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[1 - p as usize].marker.remove(iron());
    }
    Ok(())
}
