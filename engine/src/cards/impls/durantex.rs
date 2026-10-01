//! Durant ex (SSP 4): Sudden Shearing — when you play this Pokémon from your
//! hand onto your Bench, you may discard the top card of your opponent's
//! deck. Vengeful Crush — 120+; 30 more for each Prize card your opponent
//! has taken.
//!
//! Twinleaf: any PlayPokemonEffect for this card asks (ConfirmPrompt) unless
//! the Ability is blocked, even with an empty opposing deck; Vengeful Crush
//! sets `effect.damage = attack.damage + opponent.prizesTaken * 30`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Durantex", mask: mask(&[k::PLAY_POKEMON, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
            return Ok(());
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, attack, .. } = *g.e(e) {
            let base = crate::engine::attack::attack_def(g, attack).damage;
            let taken = g.st.players[opp as usize].prizes_taken;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = base + taken * 30;
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    if !results.first().copied().unwrap_or(Res::Null).as_bool() {
        return Ok(());
    }
    let o = 1 - f.a[0] as usize;
    move_count_from(g, ListRef::Deck(o as u8), ListRef::Discard(o as u8), 1, me)
}
