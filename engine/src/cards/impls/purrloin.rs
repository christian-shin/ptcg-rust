//! Purrloin (WHT): Invite Evil — search your deck for up to 3 [D] Pokémon,
//! reveal them, and put them into your hand; shuffle.
//!
//! Twinleaf: SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND with
//! { cardType: [D] } and { min: 0, max: 3 }, run on AfterAttackEffect. Phase 4b
//! (R6): with an empty deck the attack is still usable and the search does
//! nothing (the prefab throws NO_CARDS_IN_DECK, which made the attack
//! unusable).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Purrloin", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::AfterAttack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
    }
    let mut filter = Filter::none();
    filter.card_type = Some(ct::DARK);
    filter.card_type_list = true;
    search_deck_for_pokemon_to_hand(g, p, filter, ChooseCardsOpts::new(0, 3, true))
}
