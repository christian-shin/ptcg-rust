//! Search support: determinization of hidden information.
//!
//! From one player's point of view the hidden cards are their own deck order
//! and face-down prizes, and the opponent's hand, deck and prizes. A
//! determinization re-deals those cards among those positions, keeping every
//! zone's size, so a search can roll out a fully specified game.

use crate::game::{Game, GameError, R};
use crate::list::*;
use crate::rng::Rng;
use crate::state::ListRef;

/// Hidden cards of `owner` as seen by `viewer`, and their zones: the one rule for `determinize` and `Game::swap_hidden`.
fn hidden_zones(viewer: usize, owner: usize) -> (bool, bool) {
    // (hand hidden, deck+prizes hidden)
    (viewer != owner, true)
}

/// Assign hidden cards explicitly. `deck` gives the new deck order, `prizes`
/// the contents of each prize slot, `hand` the opponent's hand (ignored for
/// the viewer's own hand). The multiset of hidden cards must be unchanged.
pub fn set_hidden(g: &mut Game, viewer: usize, owner: usize, deck: &[CardId], prizes: &[Vec<CardId>], hand: Option<&[CardId]>) -> R {
    let (hand_hidden, _) = hidden_zones(viewer, owner);
    let pl = &g.st.players[owner];
    let mut before: Vec<CardId> = pl.deck.iter().collect();
    for i in 0..pl.prize_count as usize {
        before.extend(pl.prizes[i].iter());
    }
    if hand_hidden {
        before.extend(pl.hand.iter());
    }
    let mut after: Vec<CardId> = deck.to_vec();
    for p in prizes {
        after.extend(p.iter().copied());
    }
    if hand_hidden {
        after.extend(hand.ok_or(GameError("HAND_REQUIRED"))?.iter().copied());
    }
    let (mut a, mut b) = (before.clone(), after.clone());
    a.sort();
    b.sort();
    if a != b {
        return Err(GameError("HIDDEN_CARDS_MISMATCH"));
    }
    let pl = &mut g.st.players[owner];
    if deck.len() != pl.deck.len() || prizes.len() != pl.prize_count as usize {
        return Err(GameError("ZONE_SIZE_MISMATCH"));
    }
    for (i, p) in prizes.iter().enumerate() {
        if p.len() != pl.prizes[i].len() {
            return Err(GameError("ZONE_SIZE_MISMATCH"));
        }
    }
    pl.deck.set_from(deck);
    for (i, p) in prizes.iter().enumerate() {
        pl.prizes[i].set_from(p);
    }
    if hand_hidden {
        let h = hand.unwrap();
        if h.len() != pl.hand.len() {
            return Err(GameError("ZONE_SIZE_MISMATCH"));
        }
        pl.hand.set_from(h);
    }
    g.st.bump_layout();
    Ok(())
}

/// Where an unseen card lies: a position of its owner's deck, or one of their face-down Prize cards (pile, position).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unseen {
    Hand(usize),
    Deck(usize),
    Prize(usize, usize),
}

impl Game {
    /// Swap two cards that no player can see, each taking the other's place: each is in a deck or is a face-down Prize
    /// card no effect made public, both belong to the same player, and no open prompt shows the deck it is in (a search
    /// lists the deck it searches). Nothing else changes: no event, no shuffle, every zone keeps its size. Refused with
    /// `CARD_NOT_UNSEEN` otherwise (a card in a hand, in play, set aside, face up; cards of two players; a deck an open
    /// prompt lists). A determinization primitive: a host that replays a recorded game puts the cards the recording
    /// revealed where the engine will take them from; a search samples hidden cards the same way.
    pub fn swap_unseen(&mut self, a: CardId, b: CardId) -> R {
        self.swap_cards(a, b, None, "CARD_NOT_UNSEEN")
    }

    /// Swap two cards of one owner that `viewer` cannot see, under the rule [`determinize`] re-deals by
    /// (`hidden_zones`): the other player's hand, either deck, face-down Prize cards no effect made public, and no open
    /// prompt listing the deck. Each takes the other's place; no event, no shuffle, every zone keeps its size, the
    /// location hints and the layout generation follow. Refused with `CARD_NOT_HIDDEN` otherwise (a card in play, in
    /// the viewer's own hand, in a discard pile or the Lost Zone, set aside, face up; cards of two players). A host that
    /// plays one seat holds its guesses as a determinization and re-identifies one when the real card is revealed.
    pub fn swap_hidden(&mut self, viewer: usize, a: CardId, b: CardId) -> R {
        if viewer > 1 {
            return Err(GameError("CARD_NOT_HIDDEN"));
        }
        self.swap_cards(a, b, Some(viewer), "CARD_NOT_HIDDEN")
    }

    /// The shared body: `viewer` None = no player sees the cards (decks and face-down Prize cards only).
    fn swap_cards(&mut self, a: CardId, b: CardId, viewer: Option<usize>, refused: &'static str) -> R {
        let (Some(pa), Some(pb)) = (self.unseen_at(a, viewer), self.unseen_at(b, viewer)) else { return Err(GameError(refused)) };
        let owner = self.st.owner(a);
        if self.st.owner(b) != owner {
            return Err(GameError(refused));
        }
        let pu = owner as u8;
        let pl = &mut self.st.players[owner];
        for (at, c) in [(pa, b), (pb, a)] {
            match at {
                Unseen::Hand(i) => pl.hand.as_mut_slice()[i] = c,
                Unseen::Deck(i) => pl.deck.as_mut_slice()[i] = c,
                Unseen::Prize(i, k) => pl.prizes[i].as_mut_slice()[k] = c,
            }
        }
        for (at, c) in [(pa, b), (pb, a)] {
            self.st.put_at(c, match at {
                Unseen::Hand(_) => ListRef::Hand(pu),
                Unseen::Deck(_) => ListRef::Deck(pu),
                Unseen::Prize(i, _) => ListRef::Prize(pu, i as u8),
            });
        }
        self.st.bump_layout();
        Ok(())
    }

    /// The place of card `c` when no player can see it ([`Game::swap_unseen`]).
    /// With a `viewer`, hidden from that viewer by `hidden_zones` (the owner's hand too when it is not the viewer's).
    fn unseen_at(&self, c: CardId, viewer: Option<usize>) -> Option<Unseen> {
        if c >= self.st.n_cards {
            return None;
        }
        let p = self.st.owner(c);
        let pl = &self.st.players[p];
        let hand_hidden = viewer.map_or(false, |v| hidden_zones(v, p).0);
        let at = match self.st.locate(c)? {
            ListRef::Hand(q) if q as usize == p && hand_hidden => Unseen::Hand(pl.hand.index_of(c)?),
            ListRef::Deck(q) if q as usize == p => Unseen::Deck(pl.deck.index_of(c)?),
            ListRef::Prize(q, i) if q as usize == p && !pl.prize_face_up[i as usize] && !pl.prize_public[i as usize] => Unseen::Prize(i as usize, pl.prizes[i as usize].index_of(c)?),
            _ => return None,
        };
        let shown = |r: ListRef| r == ListRef::Deck(p as u8);
        let listed = self.prompts.iter().filter(|pr| !pr.answered).any(|pr| match pr.kind {
            crate::prompts::PromptKind::ChooseCards { cards, .. } | crate::prompts::PromptKind::OrderCards { cards, .. } | crate::prompts::PromptKind::AttachEnergy { cards, .. } => shown(cards),
            _ => false,
        });
        if matches!(at, Unseen::Deck(_)) && listed {
            return None;
        }
        Some(at)
    }
}

/// Uniformly random determinization for `viewer`, and a fresh chance seed.
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    let mut rng = Rng::new(seed);
    for owner in 0..2 {
        let (hand_hidden, _) = hidden_zones(viewer, owner);
        let pl = &g.st.players[owner];
        let mut pool: Vec<CardId> = pl.deck.iter().collect();
        let prize_sizes: Vec<usize> = (0..pl.prize_count as usize).map(|i| pl.prizes[i].len()).collect();
        for i in 0..pl.prize_count as usize {
            pool.extend(pl.prizes[i].iter());
        }
        let hand_len = pl.hand.len();
        if hand_hidden {
            pool.extend(pl.hand.iter());
        }
        let deck_len = pl.deck.len();
        for i in (1..pool.len()).rev() {
            pool.swap(i, rng.index(i + 1));
        }
        let mut it = pool.into_iter();
        let deck: Vec<CardId> = (&mut it).take(deck_len).collect();
        let prizes: Vec<Vec<CardId>> = prize_sizes.iter().map(|n| (&mut it).take(*n).collect()).collect();
        let hand: Vec<CardId> = if hand_hidden { (&mut it).take(hand_len).collect() } else { vec![] };
        set_hidden(g, viewer, owner, &deck, &prizes, if hand_hidden { Some(&hand) } else { None })?;
    }
    g.rng = Rng::new(rng.next_u32());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::place::tests::game_of;
    use crate::place::Zone;
    use serde_json::json;

    const MON: &str = "Hop's Phantump ASC 95";
    const POFFIN: &str = "Buddy-Buddy Poffin TEF 144";

    /// Two deck cards, or a deck card and a face-down Prize card, of one player trade places; the locations follow.
    #[test]
    fn unseen_cards_trade_places() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON}, "opp": {"reset": true, "active": MON}}));
        let p = g.st.active_player as usize;
        let deck = g.st.players[p].deck.as_slice().to_vec();
        let (a, b) = (deck[0], deck[5]);
        g.swap_unseen(a, b).unwrap();
        assert_eq!((g.st.players[p].deck.as_slice()[0], g.st.players[p].deck.as_slice()[5]), (b, a));
        assert_eq!(g.locate_card(b).unwrap().index, 0);
        assert_eq!(g.locate_card(a).unwrap().index, 5);
        let prize = g.st.players[p].prizes[2].as_slice()[0];
        g.swap_unseen(a, prize).unwrap();
        assert_eq!(g.st.players[p].prizes[2].as_slice()[0], a);
        assert_eq!(g.st.players[p].deck.as_slice()[5], prize);
        assert_eq!(g.locate_card(a).unwrap().zone, Zone::Prizes);
        assert_eq!(g.locate_card(prize).unwrap().zone, Zone::Deck);
        assert_eq!(g.st.players[p].deck.len(), deck.len());
    }

    /// A card a player can see is never swapped: a hand card, a Pokémon in play, a face-up Prize card, cards of two
    /// players, and the deck while a search's prompt lists it.
    #[test]
    fn a_seen_card_is_not_swapped() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN]}, "opp": {"reset": true, "active": MON}}));
        let p = g.st.active_player as usize;
        let top = g.st.players[p].deck.as_slice()[0];
        let refused = Err(GameError("CARD_NOT_UNSEEN"));
        let hand = g.st.players[p].hand.as_slice()[0];
        assert_eq!(g.swap_unseen(top, hand), refused);
        let active = g.st.active_pokemon(p).unwrap();
        assert_eq!(g.swap_unseen(active, top), refused);
        let theirs = g.st.players[1 - p].deck.as_slice()[0];
        assert_eq!(g.swap_unseen(top, theirs), refused);
        g.st.players[p].prize_face_up[0] = true;
        let up = g.st.players[p].prizes[0].as_slice()[0];
        assert_eq!(g.swap_unseen(top, up), refused);
        // While the search's prompt lists the deck, its cards are seen by the player searching.
        let k = g.st.players[p].hand.iter().position(|c| g.st.cdef(c).name == "Buddy-Buddy Poffin").unwrap();
        let play = crate::options::legal_actions(&g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
        g.act(play).unwrap();
        g.select().unwrap().unwrap();
        let deck = g.st.players[p].deck.as_slice().to_vec();
        assert_eq!(g.swap_unseen(deck[0], deck[1]), refused);
        let down = g.st.players[p].prizes[1].as_slice()[0];
        let other = g.st.players[p].prizes[3].as_slice()[0];
        assert_eq!(g.swap_unseen(down, other), Ok(()), "face-down Prize cards stay unseen during the search");
    }

    /// A guessed opponent hand card trades places with a card of their deck; the viewer's own hand, a public card and
    /// cards of two players are refused; a determinized game stays consistent.
    #[test]
    fn hidden_cards_trade_places_for_a_viewer() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN]}, "opp": {"reset": true, "active": MON, "hand": [POFFIN, MON]}}));
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let refused = Err(GameError("CARD_NOT_HIDDEN"));
        let (h, d) = (g.st.players[o].hand.as_slice()[0], g.st.players[o].deck.as_slice()[3]);
        let (hn, dn) = (g.st.players[o].hand.len(), g.st.players[o].deck.len());
        let gen = g.st.layout_gen;
        g.swap_hidden(v, h, d).unwrap();
        assert_eq!((g.st.players[o].hand.as_slice()[0], g.st.players[o].deck.as_slice()[3]), (d, h));
        assert_eq!(g.locate_card(d).unwrap().zone, Zone::Hand);
        assert_eq!(g.locate_card(h).unwrap().index, 3);
        assert_eq!((g.st.players[o].hand.len(), g.st.players[o].deck.len()), (hn, dn));
        assert_ne!(g.st.layout_gen, gen);
        let pz = g.st.players[o].prizes[1].as_slice()[0];
        g.swap_hidden(v, pz, d).unwrap();
        assert_eq!(g.locate_card(pz).unwrap().zone, Zone::Hand);
        // The viewer's own hand is seen by the viewer; it is hidden only from the other player.
        let mine = g.st.players[v].hand.as_slice()[0];
        let mine_deck = g.st.players[v].deck.as_slice()[0];
        assert_eq!(g.swap_hidden(v, mine, mine_deck), refused);
        assert_eq!(g.swap_hidden(o, mine, mine_deck), Ok(()));
        // Public cards: in play, cards of two owners, a face-up Prize card, a bad viewer.
        let active = g.st.active_pokemon(o).unwrap();
        let top = g.st.players[o].deck.as_slice()[0];
        assert_eq!(g.swap_hidden(v, active, top), refused);
        assert_eq!(g.swap_hidden(v, top, g.st.players[v].deck.as_slice()[1]), refused);
        g.st.players[o].prize_face_up[0] = true;
        let up = g.st.players[o].prizes[0].as_slice()[0];
        assert_eq!(g.swap_hidden(v, top, up), refused);
        assert_eq!(g.swap_hidden(2, top, top), refused);
        // Consistent after determinizing and swapping: every card is found where its list holds it.
        determinize(&mut g, v, 7).unwrap();
        let (x, y) = (g.st.players[o].hand.as_slice()[0], g.st.players[o].deck.as_slice()[0]);
        g.swap_hidden(v, x, y).unwrap();
        for (i, c) in g.st.players[o].hand.as_slice().to_vec().into_iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap().index as usize, i);
        }
        for (i, c) in g.st.players[o].deck.as_slice().to_vec().into_iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap().index as usize, i);
        }
    }
}
