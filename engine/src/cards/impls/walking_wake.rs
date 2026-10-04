//! Walking Wake (TWM): Aurora Gain — 20; heal 20 damage from this Pokémon
//! (a HealEffect on the Active). Undulating Slice — put up to 9 damage
//! counters on this Pokémon; 20 damage for each counter placed.
//!
//! Twinleaf: Undulating Slice is a non-cancellable PutDamagePrompt (90 in
//! multiples of 10, partial placement allowed, Active slot only) with a
//! per-Pokémon cap of CheckHp + 90 for every Pokémon in play; each entry is a
//! PutCountersEffect on the chosen target and `effect.damage = placed * 2`
//! (the last entry wins).
//!
//! Fixed (phase 4b, W4): the printed damage is 20 ("20×", as on the card), so
//! the resume sets `effect.damage = 0` before the entries (placing no counters
//! does 0 damage, not the printed 20).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "WalkingWake", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let pu = p as usize;
            let a = g.st.players[pu].active;
            g.run_fx(Effect::Heal { p, target: SlotRef::new(pu, a), damage: 20 })?;
        }
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
            let hp = crate::engine::check::check_hp(g, p, *s)?;
            max_allowed.push((*t, hp + 90));
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::PutDamage {
                player_type: PlayerType::BottomPlayer,
                slots,
                damage: 90,
                max_allowed,
                allow_cancel: false,
                blocked: SVec::new(),
                allow_partial: true,
                damage_multiple: 10,
            },
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
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let map: SVec<(CardTarget, i32), 16> = match results.first().copied().unwrap_or(Res::Null) {
            Res::DamageMap(m) => m,
            _ => SVec::new(),
        };
        if let Effect::Attack { damage: d, .. } = g.e_mut(atk) {
            *d = 0;
        }
        for (t, damage) in map.iter() {
            let target = get_target(&g.st, p as usize, *t)?;
            let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::PutCounters { b, damage: *damage })?;
            if let Effect::Attack { damage: d, .. } = g.e_mut(atk) {
                *d = *damage * 2;
            }
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
