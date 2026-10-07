//! Spiritomb (M5): Spiritual End — if you have 13 or more Pokémon with the
//! Hide 'n' Sneak Ability in your discard pile, choose 2 of your opponent's
//! Pokémon and quadruple the number of damage counters on each of them.
//!
//! Twinleaf: attack damage is zeroed first; the ChoosePokemonPrompt needs
//! exactly min(2, the opponent's Pokémon in play) targets (phase 4b: it needed
//! 2 even with a lone opposing Pokémon, a prompt with no valid answer); each
//! chosen Pokémon with damage gets one
//! PlaceDamageCountersEffect (source = this card) adding 3x its damage.
use super::shuppet::count_hide_n_sneak_in_discard;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Spiritomb", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
    }
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if count_hide_n_sneak_in_discard(g, p) < 13 {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        let o = 1 - p;
        let benched = g.st.players[o].bench.iter().filter(|b| !g.st.players[o].slots[**b as usize].cards.is_empty()).count();
        let count = (1 + benched).min(2) as u8;
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: count, max: count, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0];
    let targets: Vec<SlotRef> = results.first().copied().unwrap_or(Res::Null).slots().to_vec();
    for t in targets {
        let current = g.st.slot(t.p as usize, t.s).damage;
        if current <= 0 {
            continue;
        }
        g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: t, damage: current * 3, source: me })?;
    }
    Ok(())
}
