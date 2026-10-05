//! Neutralization Zone (SFA, ACE SPEC stadium): prevent all damage done to
//! Pokémon without a Rule Box by attacks from the opponent's Pokémon with a
//! Rule Box; this card can't be put into hand or deck from the discard pile.
//!
//! Twinleaf: the PutDamage branch needs the Attack phase and skips when the
//! stadium is blocked for the target; the MoveCards branch runs for any
//! discard pile holding this card (no stadium-in-play check), filtering this
//! card out of the move and preventing an emptied move.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NeutralCenter", mask: mask(&[k::USE_STADIUM, k::PUT_DAMAGE, k::MOVE_CARDS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::UseStadium { .. } => {
            if g.st.stadium_card() == Some(me) {
                bail!("CANNOT_USE_STADIUM");
            }
        }
        Effect::PutDamage { b, .. } => {
            if ignores_defender_effects(g, &b) {
                return Ok(());
            }
            if g.st.stadium_card() != Some(me) {
                return Ok(());
            }
            let owner = b.target.p as usize;
            let attacker_owner = b.source.p as usize;
            if owner == attacker_owner || g.st.phase != GamePhase::Attack || is_stadium_effect_blocked(g, owner, b.target, me) {
                return Ok(());
            }
            let src_rb = g.st.slot(b.source.p as usize, b.source.s).cards.iter().any(|c| g.st.cdef(c).has_rule_box());
            let tgt_rb = g.st.slot(b.target.p as usize, b.target.s).cards.iter().any(|c| g.st.cdef(c).has_rule_box());
            if src_rb && !tgt_rb {
                g.set_prevent(e, true);
            }
        }
        Effect::MoveCards { source, destination, cards, count, .. } => {
            for p in 0..2usize {
                if source != ListRef::Discard(p as u8) || !g.st.players[p].discard.iter().any(|c| c == me) {
                    continue;
                }
                if destination != ListRef::Hand(p as u8) && destination != ListRef::Deck(p as u8) {
                    continue;
                }
                let v: Vec<CardId>;
                let new_count;
                if let Some(cs) = cards {
                    if !cs.iter().any(|c| c == me) {
                        continue;
                    }
                    v = cs.iter().filter(|c| *c != me).collect();
                    new_count = count;
                } else if let Some(n) = count {
                    v = g.st.players[p].discard.iter().filter(|c| *c != me).take(n.max(0) as usize).collect();
                    new_count = None;
                } else {
                    v = g.st.players[p].discard.iter().filter(|c| *c != me).collect();
                    new_count = None;
                }
                let prevent = v.is_empty();
                if let Effect::MoveCards { cards, count, .. } = g.e_mut(e) {
                    *cards = Some(List::from_slice(&v));
                    *count = new_count;
                }
                if prevent {
                    g.set_prevent(e, true);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
