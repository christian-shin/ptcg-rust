//! Gholdengo (SSP 131): Strike It Rich — 30+; 90 more if this Pokémon
//! evolved from Gimmighoul during this turn. Surf Back — 100; you may shuffle
//! this Pokémon and all attached cards into your deck.
//!
//! Twinleaf: Strike It Rich only checks `pokemonPlayedTurn === state.turn` on
//! this card's list (any evolution or play this turn counts). Surf Back:
//! ConfirmPrompt, then MOVE_CARDS of the whole Active to the deck,
//! `player.active.clearEffects()` and a ShuffleDeckPrompt (no trailing wait).
//! Fixed (W1-A): Surf Back used to run in the attack handler, i.e. before the
//! damage step, so the Pokémon was already in the deck while its 100 damage
//! was dealt (no Weakness/Resistance, lost tools, a crash in handlers that
//! read the attacker); it now runs on the AfterAttackEffect, like Tuck Tail.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gholdengo", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Some(ListRef::Slot(p, s)) = g.st.locate(me) {
            if g.st.slot(p as usize, s).pokemon_played_turn == g.st.turn as i32 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 90;
                }
            }
        }
    }
    if let Effect::AfterAttack { p, attack, .. } = *g.e(e) {
        if attack == my_attack(g, me, 1) {
            let p = p as usize;
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            if !results.first().copied().unwrap_or(Res::Null).as_bool() {
                return Ok(());
            }
            let a = g.st.players[p].active;
            move_pokemon_off_board(g, SlotRef::new(p, a), ListRef::Deck(p as u8), me)?;
            let a = g.st.players[p].active;
            crate::engine::game_effect::clear_effects(&mut g.st.players[p].slots[a as usize]);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            if let Some(Res::Order(o)) = results.first() {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
