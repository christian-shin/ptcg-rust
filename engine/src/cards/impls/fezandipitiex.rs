//! Fezandipiti ex (SFA): Flip the Script — if any of your Pokémon were
//! Knocked Out during your opponent's last turn, draw 3 cards (one Flip the
//! Script per turn). Cruel Arrow — 100 damage to 1 of your opponent's Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Fezandipitiex",
    mask: mask(&[k::POWER, k::KNOCK_OUT, k::END_TURN, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn ko_marker() -> crate::markers::MarkerName {
    crate::marker!("OPPONENT_KNOCKOUT_MARKER")
}

/// `StateUtils.findOwner(state, StateUtils.findCardList(state, this))`.
fn owner_of(g: &Game, me: CardId) -> R<usize> {
    match g.st.locate(me).and_then(|l| l.owner()) {
        Some(o) => Ok(o),
        None => bail!("INVALID_GAME_STATE"),
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if !g.st.players[p].marker.has(ko_marker()) {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].used_table_turner {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 3, me)?;
        g.st.players[p].used_table_turner = true;
        ability_used(g, p, me);
        return Ok(());
    }

    match *g.e(e) {
        Effect::KnockOut { p, .. } => {
            let p = p as usize;
            // Do not activate on the player's own turn, nor between turns.
            if g.st.active_player as usize == p {
                return Ok(());
            }
            if g.st.phase != GamePhase::PlayerTurn && g.st.phase != GamePhase::Attack {
                return Ok(());
            }
            if owner_of(g, me)? == p {
                g.st.players[p].marker.add_to_state(ko_marker());
            }
            return Ok(());
        }
        Effect::EndTurn { p } => {
            let p = p as usize;
            if owner_of(g, me)? == p {
                // REMOVE_OPPONENT_LAST_TURN_MARKER_AT_END_OF_TURN (usedTurnSkip: not modeled).
                let m = &mut g.st.players[p].marker;
                if m.has(ko_marker()) {
                    m.remove(ko_marker());
                }
            }
            g.st.players[p].used_table_turner = false;
        }
        _ => {}
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = damage_opponent_pokemon(g, atk, 100, &targets);
    g.release_fx(atk);
    r
}
