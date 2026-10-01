//! Haxorus (SFA): Bring Down the Axe — if the opponent's Active has any
//! Special Energy attached it is Knocked Out (Mist-blockable
//! KnockOutOpponentEffect). Dragon Pulse — 230; discard the top 3 cards of
//! your deck (deck -> scratch CardList -> discard, two MOVE_CARDS).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Haxorus@Haxorus SFA", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            let special = g.st.slot(o, a).cards.iter().filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Special as u8
            }).count();
            if special > 0 {
                let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
                g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
                return Ok(());
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as u8;
            let top = g.alloc_temp(&[]);
            move_count_from(g, ListRef::Deck(p), top, 3, me)?;
            let n = g.lst(top).len();
            move_count_from(g, top, ListRef::Discard(p), n, me)?;
        }
    }
    Ok(())
}
