//! Maractus (JTG): Explosive Needle — if this Pokémon is in the Active Spot
//! and is Knocked Out by damage from an attack from your opponent's Pokémon,
//! put 6 damage counters on the Attacking Pokémon. Corner — 20; the
//! Defending Pokémon can't retreat during your opponent's next turn.
//!
//! Twinleaf (fixed in phase 4b, F1; rulings 1547, 1631): Explosive Needle is a Knock Out trigger (step 8 of the
//! attack flow chart). When the Knock Out is announced it adds 60 straight to the `damage` of the Pokémon that used
//! the attack, wherever it is in play then (`Game::attack_that_damaged_knocked_out`); it needs the Pokémon to have
//! been damaged by that attack while in the Active Spot, and does nothing when the attacker left play. It used to
//! predict the Knock Out while the damage was being put (step 5), before the attack's own effects.
//!
//! Fixed (phase 4b, W4): the Ability also triggered while Maractus was on the Bench and for the owner's own
//! attacks; it now needs Maractus damaged in the Active Spot by the opponent's attack.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Maractus@JTG", mask: mask(&[k::KNOCK_OUT, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::KnockOut { p, target, .. } = *g.e(e) {
        let (tp, ts) = (target.p as usize, target.s);
        if !g.st.slot(tp, ts).cards.contains(me) || g.prevented(e) {
            return Ok(());
        }
        if g.st.slot_pokemon(tp, ts) != Some(me) || is_ability_blocked(g, p as usize, me, None) {
            return Ok(());
        }
        if let Some((_, Some(src))) = g.attack_that_damaged_knocked_out(tp, target) {
            g.st.players[src.p as usize].slots[src.s as usize].damage += 60;
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        return block_retreat(g, e);
    }
    Ok(())
}
