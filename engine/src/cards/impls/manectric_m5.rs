//! Manectric (M5): Flashing Barrier — 50; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Evolution
//! Pokémon. Sonic Edge — 110; not affected by any effects on your
//! opponent's Active Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE(..., { sourceIsEvolution: true }) and
//! THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS (ApplyWeaknessEffect on the
//! attack's current damage, `damage = 0`, damage added to the opponent's
//! Active, AfterDamageEffect).
use super::mega_lopunnyex::shred;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Manectric@PBL", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let filter = PreventFilter { source_stage: Some(PreventFilter::SOURCE_IS_EVOLUTION), ..Default::default() };
        prevent_damage_filtered(g, e, filter)?;
    }
    if was_attack_used(g, e, 1, me) {
        let d = match *g.e(e) {
            Effect::Attack { damage, .. } => damage,
            _ => return Ok(()),
        };
        shred(g, e, d)?;
    }
    Ok(())
}
