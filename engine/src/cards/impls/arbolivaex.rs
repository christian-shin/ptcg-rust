//! Arboliva ex (DRI): Oil Salvo — choose 1 of your opponent's Pokémon 6
//! times; 20 damage each time, not affected by Weakness or Resistance.
//! Aroma Shot — 160; this Pokémon recovers from all Special Conditions.
//!
//! Twinleaf quirks kept: Oil Salvo is a non-cancellable PutDamagePrompt
//! (120 damage in multiples of 20, per-target cap = printed HP + 120), then
//! DAMAGE_OPPONENT_POKEMON per entry, so the Active's share goes through a
//! DealDamageEffect (Weakness/Resistance apply). Aroma Shot has no effect
//! handler (Special Conditions are not removed).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Arbolivaex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (_, c, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        max_allowed.push((*t, g.st.cdef(*c).hp + 120));
    }
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
        PromptKind::PutDamage {
            player_type: PlayerType::TopPlayer,
            slots,
            damage: 120,
            max_allowed,
            allow_cancel: false,
            blocked: SVec::new(),
            allow_partial: false,
            damage_multiple: 20,
        },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let p = match *g.e(atk) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let map: SVec<(CardTarget, i32), 16> = match results.first().copied().unwrap_or(Res::Null) {
            Res::DamageMap(m) => m,
            _ => SVec::new(),
        };
        for (t, damage) in map.iter() {
            let target = get_target(&g.st, p, *t)?;
            damage_opponent_pokemon(g, atk, *damage, &[target])?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
