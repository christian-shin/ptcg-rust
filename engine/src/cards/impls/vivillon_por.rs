//! Vivillon (POR / M3): Grand Wing — once during your turn, your opponent
//! shuffles their hand and puts it on the bottom of their deck; if they did,
//! they draw 4 cards. Blow Through — 60+; 60 more if a Stadium is in play.
//!
//! Twinleaf: throws POWER_ALREADY_USED (BIG_WINGS_MARKER) or, with an empty
//! opposing hand, CANNOT_USE_POWER. The hand is permuted in place by an
//! inline `Chance.shuffle(hand.length)`, moved into a fresh CardList, then to
//! the bottom of the deck, then MOVE_CARDS (count 4) deck → hand; the marker
//! and ABILITY_USED follow. The marker is removed at the owner's end of turn.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Vivillon@POR", mask: mask(&[k::POWER, k::ATTACK, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

fn wings() -> crate::markers::MarkerName {
    crate::marker!("BIG_WINGS_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        if g.st.players[p].marker.has_from(wings(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let n = g.st.players[o].hand.len();
        if n == 0 {
            bail!("CANNOT_USE_POWER");
        }
        let mut perm = [0u8; 120];
        g.rng.shuffle(n, &mut perm);
        let copy: Vec<CardId> = g.st.players[o].hand.as_slice().to_vec();
        {
            let hand = g.st.players[o].hand.as_mut_slice();
            for i in 0..n {
                hand[i] = copy[perm[i] as usize];
            }
        }
        let temp = g.alloc_temp(&[]);
        g.run_fx(Effect::MoveCards {
            source: ListRef::Hand(o as u8),
            destination: temp,
            cards: None,
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
        g.run_fx(Effect::MoveCards {
            source: temp,
            destination: ListRef::Deck(o as u8),
            cards: None,
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
        move_count_from(g, ListRef::Deck(o as u8), ListRef::Hand(o as u8), 4, me)?;
        ability_used(g, p, me);
        g.st.players[p].marker.add(wings(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        if g.st.stadium_card().is_some() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(wings(), me) {
            m.remove_from(wings(), me);
        }
    }
    Ok(())
}
