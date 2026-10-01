//! Sylveon ex (SSP, Tera): Magical Charm - 160; during your opponent's next
//! turn, the Defending Pokémon's attacks do 100 less damage. Angelite - choose
//! 2 of your opponent's Benched Pokémon; they shuffle those Pokémon and all
//! attached cards into their deck; can't be used if 1 of your Pokémon used
//! Angelite during your last turn. Tera: no attack damage while on the Bench.
//!
//! Twinleaf quirks kept: Angelite does nothing (and sets no marker) with an
//! empty opposing Bench; with the marker it throws BLOCKED_BY_EFFECT; the
//! marker is added before the prompt and again in its callback; each chosen
//! Pokémon is moved to the opponent's deck and followed by its own
//! ShuffleDeckPrompt for the opponent (no trailing wait). The marker pair
//! (ANGELITE / CLEAR_ANGELITE) is kept on the player and cleared at the end
//! of the following turn of that player.
use super::chikorita_asc::defending_pokemon_does_less_damage;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Sylveonex", mask: mask(&[k::ATTACK, k::END_TURN, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn angelite() -> crate::markers::MarkerName {
    crate::marker!("ANGELITE_MARKER")
}

fn clear_angelite() -> crate::markers::MarkerName {
    crate::marker!("CLEAR_ANGELITE_MARKER")
}

fn add(g: &mut Game, p: usize, m: crate::markers::MarkerName, me: CardId) {
    g.st.players[p].marker.add(m, me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        let p = p as usize;
        if g.st.players[p].marker.has_from(clear_angelite(), me) {
            g.st.players[p].marker.remove_from(angelite(), me);
            g.st.players[p].marker.remove_from(clear_angelite(), me);
        }
        if g.st.players[p].marker.has_from(angelite(), me) {
            add(g, p, clear_angelite(), me);
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        return defending_pokemon_does_less_damage(g, e, 100);
    }

    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let has_bench = g.st.players[o].bench.iter().any(|s| !g.st.players[o].slots[*s as usize].cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        if g.st.players[p].marker.has_from(angelite(), me) {
            bail!("BLOCKED_BY_EFFECT");
        }
        add(g, p, angelite(), me);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 2, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    add(g, p, angelite(), me);
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    for t in targets {
        move_pokemon_off_board(g, t, ListRef::Deck(o as u8), me)?;
        let id = g.player_id(o);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: o as u8 });
    }
    Ok(())
}
