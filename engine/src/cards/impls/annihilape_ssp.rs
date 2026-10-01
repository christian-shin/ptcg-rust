//! Annihilape (SSP): Tantrum — 130; this Pokémon is now Confused. Destined
//! Fight — both Active Pokémon are Knocked Out.
//!
//! Twinleaf (surging-sparks file): Tantrum is ADD_CONFUSION_TO_PLAYER_ACTIVE
//! (an AddSpecialConditionsPowerEffect on `player.active`); Destined Fight
//! reduces KnockOutPlayerEffect on `player.active` (the opponent takes the
//! Prizes) and then KnockOutOpponentEffect on `opponent.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Annihilape@SSP", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            add_special_conditions_to_player_active(g, p as usize, me, &[SpecialCondition::Confused])?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let mine = SlotRef::new(p as usize, g.st.players[p as usize].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: mine };
        g.run_fx(Effect::KnockOutPlayer { b, knocked_out: false, prize_count: 0 })?;
        let theirs = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: theirs };
        g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
    }
    Ok(())
}
