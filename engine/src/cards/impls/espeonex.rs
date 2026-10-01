//! Espeon ex (PRE, Tera): Psych Out - 160; discard 1 random card from your
//! opponent's hand. Amethyst - devolve each of your opponent's evolved
//! Pokémon by shuffling the highest Stage Evolution card into their deck.
//! Tera: no attack damage while on the Bench.
//!
//! Twinleaf quirks kept: the random discard uses `Chance.index`; Amethyst's
//! ShuffleDeckPrompt belongs to the *attacking* player and its order is
//! applied to the attacker's deck (the opponent's deck, which received the
//! Evolution cards, is not shuffled); the prompt has no trailing wait.
use super::strange_timepiece::devolve_pokemon;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Espeonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

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
    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
            if g.st.slot_pokemons(o, *s).len() > 1 {
                devolve_pokemon(g, SlotRef::new(o, *s), ListRef::Deck(o as u8))?;
            }
        }
        let id = g.player_id(p);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
        return Ok(());
    }
    tera_rule(g, e, me);
    Ok(())
}
