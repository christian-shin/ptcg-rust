//! Telepathic Psychic Energy (Telepath [P] Energy M3 / POR): provides [P].
//! When attached from your hand to a [P] Pokémon, search your deck for up to
//! 2 Basic [P] Pokémon and put them onto your Bench, then shuffle.
//!
//! Twinleaf reacts to every AttachEnergyEffect of this card (not only from
//! the hand); the type check runs before the card is attached. Phase 4b: the search
//! always happens when the deck is not empty (it used to be skipped, without a
//! shuffle, when the deck held no Basic [P] Pokémon).
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "TelepathPsychicEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::ATTACH_ENERGY]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckProvidedEnergy { source, .. } = *g.e(e) {
        if g.st.slot(source.p as usize, source.s).cards.contains(me) {
            let mut provides = SVec::new();
            provides.push(ct::PSYCHIC);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
    }

    if let Effect::AttachEnergy { p, card, target } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        if is_special_energy_blocked(g, p, me, target, false) {
            return Ok(());
        }
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        if !matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC)) {
            return Ok(());
        }
        let slots = empty_bench_slots(g, p).len();
        if slots == 0 {
            return Ok(());
        }
        // Fixed (phase 4b, rulings 336/779/1764): an empty deck can't be searched (public); a deck without a Basic [P]
        // Pokémon is still searched and shuffled afterwards (every search shuffles), the player just finds nothing.
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let filter = Filter { stage: Some(Stage::Basic as u8), card_type: Some(ct::PSYCHIC), card_type_list: true, ..Filter::none() };
        search_deck_for_pokemon_to_bench(g, p, filter, ChooseCardsOpts::new(0, 2, true))?;
    }
    Ok(())
}
