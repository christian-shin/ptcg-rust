//! Eevee ex (PRE): Rainbow DNA — you can play Pokémon ex that evolve from
//! Eevee onto this Pokémon to evolve it. Coruscating Quartz — 200.
//! Tera: prevent attack damage to this Pokémon while it is on the Bench.
//!
//! Twinleaf: on every CheckTableStateEffect this card sets its own
//! `evolvesFromBase` to `['Eevee']` when it is in play and a stub-Ability
//! probe for its owner passes, else `[]`. Any card whose `evolvesFrom` is
//! 'Eevee' (ex or not) can then evolve it.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eeveeex", mask: mask(&[k::CHECK_TABLE_STATE, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

static EEVEE: &[&str] = &["Eevee"];
static NONE: &[&str] = &[];

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckTableState { .. } = *g.e(e) {
        let v = match g.st.find_pokemon_slot(me) {
            None => NONE,
            Some((owner, _)) => {
                if is_ability_blocked(g, owner, me, None) {
                    NONE
                } else {
                    EEVEE
                }
            }
        };
        g.st.cards[me as usize].evolves_from_base = Some(v);
    }
    tera_rule(g, e, me);
    Ok(())
}
