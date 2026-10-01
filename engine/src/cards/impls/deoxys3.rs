//! Deoxys (M4 33): Psy Protection - 80; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Pokémon that have
//! an Ability.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceHasAbility: true }` (damage only).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Deoxys3", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let filter = PreventFilter { source_has_ability: true, ..Default::default() };
        prevent_damage_filtered(g, e, filter)?;
    }
    Ok(())
}
