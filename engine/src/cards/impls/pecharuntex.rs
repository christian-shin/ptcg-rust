//! Pecharunt ex (SFA): Subjugating Chains - once during your turn, switch 1
//! of your Benched [D] Pokémon (except Pecharunt ex) with your Active; the
//! new Active is Poisoned. Irritated Outburst - 60x prizes the opponent took.
//!
//! Twinleaf quirks kept: the ABILITY_USED board effect is placed before the
//! checks run; `player.active.clearEffects()` runs on the old Active before
//! the switch; `chainsOfControlUsed` is a player field reset at every
//! EndTurnEffect; CheckTableState sets `pecharuntexIsInPlay` (never cleared)
//! when a player with this card in play has a Special Condition on the Active.
use crate::cards::prelude::*;
use crate::engine::game_effect::clear_effects;

pub static IMPL: CardImpl = CardImpl {
    class: "Pecharuntex",
    mask: mask(&[k::END_TURN, k::POWER, k::ATTACK, k::CHECK_TABLE_STATE]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn is_dark(g: &Game, c: CardId) -> bool {
    g.st.cdef(c).card_type.contains(&ct::DARK)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].chains_of_control_used = false;
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        ability_used(g, p, me);
        let has_dark_bench = g.st.players[p]
            .bench
            .iter()
            .any(|b| g.st.slot_pokemon(p, *b).map(|c| is_dark(g, c) && g.st.cdef(c).name != "Pecharunt ex").unwrap_or(false));
        if g.st.players[p].chains_of_control_used {
            bail!("POWER_ALREADY_USED");
        }
        if !has_dark_bench {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let mut blocked = TargetList::new();
        let mut dark = 0;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
            if g.st.cdef(*c).name == "Pecharunt ex" {
                blocked.push(*t);
            }
            if !is_dark(g, *c) {
                blocked.push(*t);
            }
            if is_dark(g, *c) {
                dark += 1;
            }
        }
        if dark <= 1 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_SWITCH",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            let taken = g.st.players[opp as usize].prizes_taken;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = taken * 60;
            }
        }
    }

    if let Effect::CheckTableState { .. } = *g.e(e) {
        for p in 0..2 {
            let a = g.st.players[p].active;
            if g.st.slot(p, a).special_conditions.is_empty() {
                continue;
            }
            if for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
                g.st.players[p].pecharuntex_is_in_play = true;
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel = first.slots();
    if let Some(t) = sel.first() {
        let t = *t;
        let a = g.st.players[p].active;
        clear_effects(&mut g.st.players[p].slots[a as usize]);
        if t.p as usize == p {
            crate::engine::turn::switch_pokemon(g, p, t.s)?;
        }
        let a = g.st.players[p].active;
        crate::engine::phase::add_condition(&mut g.st.players[p].slots[a as usize], SpecialCondition::Poisoned);
        g.st.players[p].chains_of_control_used = true;
    }
    Ok(())
}
