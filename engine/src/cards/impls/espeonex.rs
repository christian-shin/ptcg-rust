//! Espeon ex (PRE, Tera): Psych Out - 160; discard 1 random card from your
//! opponent's hand. Amethyst - devolve each of your opponent's evolved
//! Pokémon by shuffling the highest Stage Evolution card into their deck.
//! Tera: no attack damage while on the Bench.
//!
//! Twinleaf: the random discard uses `Chance.index`; the ShuffleDeckPrompt
//! has no trailing wait.
//!
//! Fixed (phase 4b, W4): Amethyst's ShuffleDeckPrompt belonged to the
//! attacking player and its order was applied to the attacker's deck, leaving
//! the opponent's deck (which received the Evolution cards) unshuffled; it now
//! shuffles the opponent's deck. Resistance is Fighting -30 (was -20).
use super::strange_timepiece::devolve_pokemon;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Espeonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[o].hand.len();
        if n > 0 {
            let i = g.rng.index(n);
            let c = g.st.players[o].hand.as_slice()[i];
            move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &[c], me)?;
        }
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
            if g.st.slot_pokemons(o, *s).len() > 1 {
                // An effect of the attack on that Pokémon: Mist Energy and the like prevent it.
                let (p, attack, source) = match *g.e(e) {
                    Effect::Attack { p, attack, source, .. } => (p, attack, source),
                    _ => return Ok(()),
                };
                let b = AtkBase { attack_effect: e, player: p, opponent: o as u8, attack, source, target: SlotRef::new(o, *s) };
                let (_, prevented) = g.run_fx(Effect::Devolve { b })?;
                if !prevented {
                    devolve_pokemon(g, SlotRef::new(o, *s), ListRef::Deck(o as u8))?;
                }
            }
        }
        let id = g.player_id(o);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: o as u8 });
        return Ok(());
    }
    tera_rule(g, e, me);
    Ok(())
}
