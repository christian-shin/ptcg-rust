//! Metagross (M4 / CRI): Bounce Back — 60, after attacking you choose 1 of
//! your opponent's Benched Pokémon to switch in (Twinleaf lets the attacker
//! choose). Metallic Hammer — 150+, you may discard 3 [M] Energy from this
//! Pokémon for 150 more damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Metagross@Metagross M4",
    mask: mask(&[k::ATTACK, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        switch_in_opponent_benched_pokemon(g, p, false);
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let (p, source) = match *g.e(e) {
            Effect::Attack { p, source, .. } => (p as usize, source),
            _ => return Ok(()),
        };
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source, energy_map: SVec::new() })?;
        let metal = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().filter(|m| m.provides.contains(&ct::METAL)).count(),
            _ => 0,
        };
        if metal >= 3 {
            g.retain_fx(e);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.e[0] = e;
            confirmation_prompt(g, p, "WANT_TO_DISCARD_ENERGY", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

/// `energyCardProvidesType(card, METAL)`.
fn provides_metal(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_energy() && (d.provides.contains(&ct::METAL) || d.provides.contains(&ct::ANY))
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                g.release_fx(atk);
                return Ok(());
            }
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage += 150;
            }
            // DISCARD_UP_TO_X_TYPE_ENERGY_FROM_YOUR_POKEMON(effect, 3, M, 3, [ACTIVE]).
            let mut available = 0;
            let mut o = MoveOpts { allow_cancel: false, ..Default::default() };
            for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if t.slot != SlotType::Active {
                    continue;
                }
                let mut b = Blocked::default();
                for (i, c) in g.st.slot(p, s).cards.iter().enumerate() {
                    if provides_metal(g, c) {
                        available += 1;
                    } else {
                        b.push(i as u8);
                    }
                }
                o.blocked_map.push((t, b));
            }
            if available == 0 {
                g.release_fx(atk);
                return Ok(());
            }
            let max = 3.min(available) as u8;
            o.min = 3.min(max);
            o.max = Some(max);
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.e[0] = atk;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_ENERGIES_TO_DISCARD",
                PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                Cont::Card { card: _me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let r = discard_chosen(g, p, atk, first);
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}

fn discard_chosen(g: &mut Game, p: usize, atk: EffId, first: Res) -> R {
    let transfers = match first {
        Res::CardsFrom(t) => t,
        _ => return Ok(()),
    };
    if transfers.is_empty() {
        return Ok(());
    }
    if !transfers.iter().all(|(_, c)| provides_metal(g, *c)) {
        bail!("INVALID_PROMPT_RESULT");
    }
    // discardTransfersAsEffects: one DiscardCardsEffect per source, in first-seen order.
    let mut groups: Vec<(SlotRef, SVec<CardId, 16>)> = Vec::new();
    for (from, c) in transfers.iter().copied() {
        let s = get_target(&g.st, p, from)?;
        match groups.iter_mut().find(|(x, _)| *x == s) {
            Some((_, v)) => v.push(c),
            None => {
                let mut v = SVec::new();
                v.push(c);
                groups.push((s, v));
            }
        }
    }
    let (pp, opp, attack, source) = match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    for (s, cards) in groups {
        let b = AtkBase { attack_effect: atk, player: pp, opponent: opp, attack, source, target: s };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}
