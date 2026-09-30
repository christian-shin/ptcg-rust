//! Genesect ex (BLK / SV11B): Metal Signal — once during your turn, search
//! your deck for up to 2 [M] Evolution Pokémon, reveal them, put them into
//! your hand, then shuffle. Protect Charge — 150; during your opponent's next
//! turn this Pokémon takes 30 less damage from attacks.
//!
//! Twinleaf: the once-per-turn marker lives on the player (per card); the
//! blocked indices are computed on the unsorted deck ("Evolution" = non-empty
//! `evolvesFrom`, not LV.X). Protect Charge writes `damageReductionNextTurn`
//! on the player's Active (whatever it is).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Genesectex",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn metal_signal() -> crate::markers::MarkerName {
    crate::marker!("METAL_SIGNAL_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(metal_signal(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(metal_signal(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut opts = ChooseCardsOpts::new(0, 2, false);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            let ok = d.is_pokemon() && d.card_type.contains(&ct::METAL) && !d.evolves_from.is_empty() && d.stage != Stage::LvX as u8;
            if !ok {
                opts.blocked.push(i as u8);
            }
        }
        search_deck_for_cards_to_hand(g, p, me, Filter::super_type(SuperType::Pokemon), opts);
        ability_used(g, p, me);
        g.st.players[p].marker.add(metal_signal(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }

    remove_marker_at_end_of_turn(g, e, metal_signal(), me);

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        g.st.players[p].slots[a as usize].damage_reduction_next_turn = 30;
    }
    Ok(())
}
