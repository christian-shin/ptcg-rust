//! Usage records (events batch 9 C6): "once during your turn", "you can't use more than 1 X Ability each turn", "once per
//! game", a Tool's once, and "the effect doesn't stack" are one rules concept, a record in `State::usage` ([`Usage`]).
//!
//! - A Pokémon's own use of an Ability ("once during your turn", per copy) is a record of the card (`UsageBy::Card`) keyed
//!   by the Ability's printed name; it ends with the turn, and with the Pokémon: entering or leaving play, evolving or
//!   devolving makes a new Pokémon (id317; [`reset_card`]); "information on which Abilities have already been used also remains
//!   on the new Pokémon" when a card is switched with another (APR D-19; [`abilities_of`], [`carry`]).
//! - "You can't use more than 1 X Ability each turn" is the player's (`UsageBy::Player`), keyed by the same printed name; a
//!   Pokémon entering play again doesn't reset it (batch 8 rules fix).
//! - "Once per game" and a Tool's once are the player's, keyed by the declaration (`UsageKey::Declared`).
//! - "Doesn't stack" (Wonder Kiss, Shadowy Concealment: "the effect doesn't stack", id2083) is a record of the Pokémon the
//!   effect is about, for as long as it is in play.

use crate::list::CardId;
use crate::state::{State, Until, Usage, UsageBy, UsageKey};

/// The key of the printed Ability (or Pokémon Power) number `index` of `card`.
pub fn ability_key(st: &State, card: CardId, index: u8) -> UsageKey {
    match st.cdef(card).powers.get(index as usize) {
        Some(p) => UsageKey::Ability(p.name_id),
        None => UsageKey::Declared { def: st.cards[card as usize].def, index: 0x80 | index },
    }
}

/// The key of the card's own declaration number `index` (a passive: its position in the card's passives).
pub fn declared_key(st: &State, card: CardId, index: u8) -> UsageKey {
    UsageKey::Declared { def: st.cards[card as usize].def, index }
}

/// Is there a record of `key` by `by`?
#[inline]
pub fn has(st: &State, key: UsageKey, by: UsageBy) -> bool {
    st.usage.iter().any(|u| u.key == key && u.by == by)
}

/// The record is made: a card's Ability record sets the card's bit of `State::usage_held`.
#[inline]
fn push(st: &mut State, key: UsageKey, by: UsageBy, until: Until) {
    if let (UsageBy::Card(c), UsageKey::Ability(_)) = (by, key) {
        st.usage_held[(c >> 6) as usize] |= 1u64 << (c & 63);
    }
    st.usage.push(Usage { key, by, until });
}

/// Make the record (one of a key and holder at most).
pub fn record(st: &mut State, key: UsageKey, by: UsageBy, until: Until) {
    if !has(st, key, by) {
        push(st, key, by, until);
    }
}

/// Take the record, or refuse when it is there already ("once during your turn").
pub fn use_once(st: &mut State, key: UsageKey, by: UsageBy, until: Until) -> bool {
    if has(st, key, by) {
        return false;
    }
    push(st, key, by, until);
    true
}

/// The records that end with player `p`'s turn, before its end-of-turn effects (`Until::ThisTurn`).
pub fn expire_this_turn(st: &mut State, p: usize) {
    if !st.usage.is_empty() {
        st.usage.retain(|u| u.until != Until::ThisTurn(p as u8));
    }
}

/// A Pokémon card that enters or leaves play, or is covered by an evolution, is a new Pokémon for the Abilities it used (id317):
/// its own Ability records end. The player's records stay.
pub fn reset_card(st: &mut State, card: CardId) {
    // One bit test for a card that never used an Ability (every slot move asks).
    let (w, bit) = ((card >> 6) as usize, 1u64 << (card & 63));
    if st.usage_held[w] & bit == 0 {
        return;
    }
    st.usage_held[w] &= !bit;
    st.usage.retain(|u| !(u.by == UsageBy::Card(card) && matches!(u.key, UsageKey::Ability(_))));
}

/// [`reset_card`] for every card of the spot `t` (an evolved stack is a new Pokémon: id317).
pub fn reset_slot(st: &mut State, t: crate::effects::SlotRef) {
    if st.usage_held == [0; 2] {
        return;
    }
    let mut cards: crate::list::SVec<CardId, 64> = crate::list::SVec::new();
    for c in st.players[t.p as usize].slots[t.s as usize].cards().iter() {
        cards.push(c);
    }
    for c in cards.iter().copied() {
        reset_card(st, c);
    }
}

/// The Abilities `card` has used (its own records), read before the card is moved (`carry`).
pub fn abilities_of(st: &State, card: CardId) -> crate::list::SVec<(UsageKey, Until), 4> {
    let mut out = crate::list::SVec::new();
    for u in st.usage.iter() {
        if u.by == UsageBy::Card(card) && matches!(u.key, UsageKey::Ability(_)) && out.len() < out.capacity() {
            out.push((u.key, u.until));
        }
    }
    out
}

/// A card switched with another is the same Pokémon (APR D-19): the Abilities the old card used are the new card's.
pub fn carry(st: &mut State, new: CardId, used: &[(UsageKey, Until)]) {
    for (key, until) in used {
        record(st, *key, UsageBy::Card(new), *until);
    }
}

#[cfg(test)]
mod tests {
    use crate::spec::Once;

    /// Every limited Ability of a spec card is a printed power of every printing the spec serves, so its key is the printed
    /// name (`ability_key` never falls back to the card's own declaration).
    #[test]
    fn a_limited_power_is_a_printed_ability_of_every_printing() {
        let mut checked = 0;
        for (id, d) in crate::gen::cards::CARDS.iter().enumerate() {
            let Some(spec) = crate::cards::spec_for(id as crate::carddb::DefId) else { continue };
            for pw in spec.powers.iter().filter(|pw| pw.once != Once::No) {
                let printed = d.powers.get(pw.index as usize);
                assert!(printed.is_some_and(|p| p.source() == crate::spec::passive::RuleSource::Ability), "{}: power {} is no printed Ability", d.full_name, pw.index);
                checked += 1;
            }
        }
        assert!(checked >= 30, "{checked} limited powers");
    }

    /// The same printed name is one key for every copy and printing ("you can't use more than 1 Fan Call Ability each turn").
    #[test]
    fn the_key_is_the_printed_name() {
        let names: Vec<&str> = crate::gen::cards::ABILITY_NAMES.to_vec();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names, sorted, "one id per distinct name, in order");
        for d in crate::gen::cards::CARDS {
            for p in d.powers {
                assert_eq!(crate::gen::cards::ABILITY_NAMES[p.name_id as usize], p.name);
            }
        }
    }
}
