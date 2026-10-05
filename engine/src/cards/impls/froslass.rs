//! Froslass (TWM): Freezing Shroud — during Pokémon Checkup, put 1 damage
//! counter on each Pokémon in play that has any Abilities (excluding any
//! Froslass). Frost Smash — 60.
//!
//! Twinleaf: at every EndTurnEffect, each Froslass card (wherever it is)
//! adds CHILLING_CURTAIN_MARKER (sourced by itself) to each player with an
//! in-play Froslass that has the ability, unless that player already has
//! the marker; at BetweenTurns for a player carrying the marker from this
//! card, it places 10 × (that player's Freezing Shroud Froslass count) on
//! every non-Froslass Pokémon with an Ability on both sides (the player's
//! first), then removes the marker. The ability lock check uses the marker
//! owner.
//!
//! Twinleaf fix (phase 4b, Y2-1): the Froslass count compares the ability name
//! with the literal 'Freezing Shroud'; it read `this.powers[0].name`, which
//! threw at every EndTurn for a copycat without Abilities that had copied
//! Frost Smash (Zoroark's Foul Play) while a Froslass was in play.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Froslass", mask: mask(&[k::BETWEEN_TURNS, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

fn chilling() -> crate::markers::MarkerName {
    crate::marker!("CHILLING_CURTAIN_MARKER")
}

fn count_froslass(g: &Game, p: usize) -> i32 {
    let mut n = 0;
    for (_, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let d = g.st.cdef(c);
        if d.name == "Froslass" && d.powers.iter().any(|pw| pw.name == "Freezing Shroud") {
            n += 1;
        }
    }
    n
}

fn has_ability(g: &mut Game, p: usize, c: CardId) -> R<bool> {
    let mut powers = SVec::new();
    for i in 0..g.st.cdef(c).powers.len() {
        powers.push(PowerRef { card: c, index: i as u8 });
    }
    let (e, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers })?;
    Ok(match e {
        Effect::CheckPokemonPowers { powers, .. } => {
            powers.iter().any(|r| g.st.cdef(r.card).powers[r.index as usize].power_type == PowerType::Ability as u8)
        }
        _ => false,
    })
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::BetweenTurns { p, .. } = *g.e(e) {
        let p = p as usize;
        if !g.st.players[p].marker.has_from(chilling(), me) {
            return Ok(());
        }
        if g.st.phase != GamePhase::BetweenTurns {
            return Ok(());
        }
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let o = 1 - p;
        let n = count_froslass(g, p);
        for q in [p, o] {
            for (s, c, _) in for_each_pokemon(g, q, PlayerType::BottomPlayer).iter().copied() {
                if g.st.cdef(c).name == "Froslass" {
                    continue;
                }
                if has_ability(g, q, c)? {
                    g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: SlotRef::new(q, s), damage: 10 * n, source: me })?;
                }
            }
        }
        g.st.players[p].marker.remove_from(chilling(), me);
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let p = p as usize;
        for q in [p, 1 - p] {
            if count_froslass(g, q) > 0 && !g.st.players[q].marker.has(chilling()) {
                g.st.players[q].marker.add(chilling(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            }
        }
    }
    Ok(())
}
