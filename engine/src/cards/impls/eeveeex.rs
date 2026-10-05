//! Eevee ex (PRE): Rainbow DNA — you can play Pokémon ex that evolve from
//! Eevee onto this Pokémon to evolve it. Coruscating Quartz — 200.
//! Tera: prevent attack damage to this Pokémon while it is on the Bench.
//!
//! Twinleaf: on every CheckTableStateEffect this card sets its own
//! `evolvesFromBase` to `['Eevee']` when it is in play and a stub-Ability
//! probe for its owner passes, else `[]`, so any card whose `evolvesFrom` is
//! 'Eevee' could evolve it. Fixed (R1-4): a PlayPokemonEffect whose target
//! holds this card as top Pokémon throws INVALID_TARGET when the played card
//! evolves from 'Eevee' and is not a Pokémon ex (Sylveon PRE 40 no longer can).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eeveeex", mask: mask(&[k::CHECK_TABLE_STATE, k::PUT_DAMAGE, k::PLAY_POKEMON]), reduce, resume: None, coin: None, can_play: None };

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
    // Rainbow DNA: only a Pokémon ex that evolves from Eevee can evolve this Pokémon.
    if let Effect::PlayPokemon { card, target, .. } = *g.e(e) {
        if g.st.slot_pokemon(target.p as usize, target.s) == Some(me) {
            let d = g.st.cdef(card);
            if d.evolves_from == "Eevee" && !d.has_tag(tag::POKEMON_EX_LOWER) {
                bail!("INVALID_TARGET");
            }
        }
    }
    tera_rule(g, e, me);
    Ok(())
}
