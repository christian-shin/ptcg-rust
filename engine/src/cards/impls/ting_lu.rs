//! Ting-Lu (TWM 110): Ground Crack — 30; if a Stadium is in play, 30 damage
//! to each of your opponent's Benched Pokémon, then discard that Stadium.
//! Hammer In — 110.
//!
//! Fixed (phase 4b, R7F-2; rulings 1559, 1589): Twinleaf set a card-object
//! flag in the attack and discarded the Stadium on a later BetweenTurnsEffect,
//! after the Knock Out check (a Pokémon the Stadium's HP bonus or Tool lock
//! kept alive was Knocked Out) and, when the Stadium had already left play,
//! the flag stayed set and discarded a later Stadium. The Stadium is now
//! discarded in AfterAttackEffect: after the damage, before the Knock Outs.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TingLu", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if g.st.stadium_card().is_none() {
            return Ok(());
        }
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
            if s == g.st.players[o].active {
                continue;
            }
            put_damage(g, e, 30, SlotRef::new(o, s))?;
        }
    }
    if after_attack_used(g, e, 0, me) {
        if let Some(stadium) = g.st.stadium_card() {
            if let Some(l) = g.st.locate(stadium) {
                let owner = l.owner().unwrap_or(0);
                g.run_fx(Effect::MoveCards {
                    source: l,
                    destination: ListRef::Discard(owner as u8),
                    cards: None,
                    count: None,
                    to_top: false,
                    to_bottom: false,
                    skip_cleanup: false,
                    source_card: me,
                })?;
            }
        }
    }
    Ok(())
}
