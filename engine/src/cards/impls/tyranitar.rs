//! Tyranitar (JTG): Daunting Gaze — while this Pokémon is in the Active
//! Spot, your opponent can't play Item cards from their hand. Crackling
//! Stomp — 150; discard the top 2 cards of your opponent's deck.
//!
//! Twinleaf: every PlayItemEffect is inspected while this card is in play
//! on either side. If it is the item player's own Active (and not the
//! opponent's) nothing happens; if it is the opponent's Active, a real
//! PowerEffect for the *item player* probes the ability lock and
//! BLOCKED_BY_ABILITY is thrown when it passes.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Tyranitar", mask: mask(&[k::ATTACK, k::PLAY_ITEM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(o as u8), ListRef::Discard(o as u8), 2, me)?;
    }
    if let Effect::PlayItem { p, .. } = *g.e(e) {
        let p = p as usize;
        let o = 1 - p;
        let in_play = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|x| x.1 == me)
            || for_each_pokemon(g, o, PlayerType::TopPlayer).iter().any(|x| x.1 == me);
        if !in_play {
            return Ok(());
        }
        let mine_active = g.st.active_pokemon(p) == Some(me);
        let opp_active = g.st.active_pokemon(o) == Some(me);
        if mine_active && !opp_active {
            return Ok(());
        }
        if opp_active {
            let power = PowerRef { card: me, index: 0 };
            if g.run_fx(Effect::Power { p: p as u8, power, card: me, target: None, probe: false }).is_err() {
                return Ok(());
            }
            bail!("BLOCKED_BY_ABILITY");
        }
    }
    Ok(())
}
