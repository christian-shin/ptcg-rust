//! Punk Helmet (PFL, tool): if the [D] Pokémon this card is attached to is in
//! the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if this Pokémon is Knocked Out), place 4 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: on AfterDamageEffect against the holder: needs damage above 0,
//! an opposing attacker and the holder in the Active Spot; then the tool
//! probe (a bare ToolEffect for the attacking player) and a
//! CheckPokemonTypeEffect on the holder; with Dark type during the attack
//! phase `source.damage += 40` directly (no damage effect, no KO check).
//!
//! Step 7 of the attack flow chart (F1): the damage records the trigger and it resolves after the attack's own
//! effects (AttackTrigger): the Tool must still be attached (ruling 1649) and not blocked, and the Attacking
//! Pokémon still in play (ruling 530); on the Bench it still gets the counters (ruling 1839).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PunkHelmet", mask: mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AfterDamage { b, damage } => {
            let t = b.target;
            if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
                return Ok(());
            }
            if damage <= 0 || b.player == t.p || g.st.players[t.p as usize].active != t.s {
                return Ok(());
            }
            g.attack_trigger(b, damage, me, None)
        }
        Effect::AttackTrigger { p, card, target: t, source, source_in_play, retaliate: None, .. } if card == me => {
            if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Tool { p, card: me }).is_err() {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, t);
            let (ce, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
            let dark = matches!(ce, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::DARK));
            if dark && g.st.phase == GamePhase::Attack && source_in_play {
                g.st.players[source.p as usize].slots[source.s as usize].damage += 40;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
