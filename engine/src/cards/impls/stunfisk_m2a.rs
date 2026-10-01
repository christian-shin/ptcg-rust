//! Stunfisk (M2a): Pouncing Trap — 30; during your opponent's next turn the
//! Defending Pokémon can't retreat. During your next turn, the Defending
//! Pokémon takes 100 more damage from attacks (after Weakness/Resistance).
//!
//! Twinleaf: DEFENDING_POKEMON_TAKES_MORE_DAMAGE_DURING_YOUR_NEXT_TURN(100)
//! (a DefendingPokemonTakesMoreDamageDuringAttackerNextTurnEffect on the
//! opponent's Active), then BLOCK_RETREAT (PreventRetreatEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Stunfisk@ASC", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let b = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => {
                let o = opp as usize;
                let target = SlotRef::new(o, g.st.players[o].active);
                AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
            }
            _ => return Ok(()),
        };
        g.run_fx(Effect::DefendingPokemonTakesMoreDamage { b, damage_bonus: 100 })?;
        block_retreat(g, e)?;
    }
    Ok(())
}
