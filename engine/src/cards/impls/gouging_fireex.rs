//! Gouging Fire ex (TEF): Heat Blast — 60. Blaze Blitz — 260; this Pokémon
//! can't use Blaze Blitz again until it leaves the Active Spot
//! (PREVENT_THIS_ATTACK_UNTIL_LEAVES_ACTIVE: a PreventAttackUntilLeavesActive
//! EffectOfAttack that sets `source.blockedAttackNameUntilLeavesActive`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GougingFireex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        // `this.attacks[1].name`: under a copy-attack session `this.attacks`
        // are the copied clones (Ethan's Sudowoodo's Try to Imitate).
        let name = crate::engine::attack::attack_def(g, my_attack(g, me, 1)).name;
        let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::PreventAttackUntilLeavesActive { b, name })?;
    }
    Ok(())
}
