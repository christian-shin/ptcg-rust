//! Jolteon ex (PRE, Tera): Flashing Spear — 60+; you may discard up to 2
//! Basic Energy from your Benched Pokémon, 90 more damage for each card
//! discarded. Dravite — 280; during your next turn this Pokémon can't attack.
//!
//! Twinleaf: Flashing Spear resets `damage = 60`, then
//! DISCARD_UP_TO_X_ENERGY_FROM_YOUR_POKEMON(2, { energyType: BASIC }, 0,
//! [BENCH]) — no prompt without Basic Energy on the Bench; one
//! DiscardCardsEffect per source slot (first-seen order), then
//! `damage = 60 + 90 * transfers`. Dravite: THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Jolteonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 60;
        }
        let mut available = 0usize;
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if t.slot != SlotType::Bench {
                continue;
            }
            available += g.st.slot(p, s).cards.iter().filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Basic as u8
            }).count();
        }
        if available == 0 {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        let o = MoveOpts { allow_cancel: false, min: 0, max: Some(available.min(2) as u8), ..Default::default() };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    tera_rule(g, e, me);
    Ok(())
}

/// `discardTransfersAsEffects`: one DiscardCardsEffect per source slot, in
/// first-seen order.
pub fn discard_transfers_as_effects(g: &mut Game, p: usize, e: EffId, transfers: &[(CardTarget, CardId)]) -> R {
    let mut groups: Vec<(SlotRef, SVec<CardId, 64>)> = Vec::new();
    for (from, c) in transfers.iter() {
        let s = get_target(&g.st, p, *from)?;
        match groups.iter_mut().find(|(t, _)| *t == s) {
            Some((_, v)) => v.push(*c),
            None => {
                let mut v = SVec::new();
                v.push(*c);
                groups.push((s, v));
            }
        }
    }
    let (opp, attack, source) = match *g.e(e) {
        Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
        _ => return Ok(()),
    };
    for (target, cards) in groups {
        let b = AtkBase { attack_effect: e, player: p as u8, opponent: opp, attack, source, target };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let e = f.e[0];
    let r = (|| -> R {
        let transfers = match results.first().copied() {
            Some(Res::CardsFrom(t)) => t,
            _ => return Ok(()),
        };
        if transfers.is_empty() {
            return Ok(());
        }
        discard_transfers_as_effects(g, p, e, transfers.as_slice())?;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 60 + transfers.len() as i32 * 90;
        }
        Ok(())
    })();
    g.release_fx(e);
    r
}
