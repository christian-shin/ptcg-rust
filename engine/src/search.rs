//! Search support: determinization of hidden information.
//!
//! From one player's point of view the hidden cards are their own deck order
//! and face-down prizes, and the opponent's hand, deck and prizes. A
//! determinization re-deals those cards among those positions, keeping every
//! zone's size, so a search can roll out a fully specified game. When the
//! viewer doesn't know which cards those are (the other player's deck list),
//! `Game::set_unseen` gives them identities from a card multiset first.

use crate::carddb::DefId;
use crate::cause::Cause;
use crate::game::{Game, GameError, R};
use crate::list::*;
use crate::rng::Rng;
use crate::state::{LastingWhat, ListRef, MAX_CARDS};

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
    // The location hints follow the cards (a stale hint costs a scan, never a wrong answer).
    let pu = owner as u8;
    let moved = deck.iter().map(|c| (*c, ListRef::Deck(pu)))
        .chain(prizes.iter().enumerate().flat_map(|(i, v)| v.iter().map(move |c| (*c, ListRef::Prize(pu, i as u8)))))
        .chain(hand.filter(|_| hand_hidden).into_iter().flatten().map(|c| (*c, ListRef::Hand(pu))));
    for (c, r) in moved {
        g.st.put_at(c, r);
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

/// Uniformly random determinization for `viewer`, and a fresh chance seed. Only cards the viewer cannot see move
/// (`Game::unseen_at` with the viewer, the rule `Game::swap_hidden` uses): a face-up or public Prize card, a deck an
/// open prompt lists and every card outside hand, deck and Prize cards stay where they are; the hidden cards of each
/// zone are re-dealt among the hidden places, keeping every zone's size.
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    let mut rng = Rng::new(seed);
    for owner in 0..2 {
        redeal(g, viewer, owner, &mut rng)?;
    }
    g.rng = Rng::new(rng.next_u32());
    Ok(())
}

/// The places of `owner`'s cards hidden from `viewer` (`Game::unseen_at`), in a fixed order (deck, Prize cards, hand),
/// and the cards in them.
fn hidden_places(g: &Game, viewer: usize, owner: usize) -> (Vec<Unseen>, Vec<CardId>) {
    let pl = &g.st.players[owner];
    let mut places: Vec<Unseen> = vec![];
    let mut cards: Vec<CardId> = vec![];
    let all = pl.deck.iter().enumerate().map(|(i, c)| (c, Unseen::Deck(i)))
        .chain((0..pl.prize_count as usize).flat_map(|i| pl.prizes[i].iter().enumerate().map(move |(k, c)| (c, Unseen::Prize(i, k)))))
        .chain(pl.hand.iter().enumerate().map(|(i, c)| (c, Unseen::Hand(i))));
    for (c, at) in all {
        if g.unseen_at(c, Some(viewer)) == Some(at) {
            places.push(at);
            cards.push(c);
        }
    }
    (places, cards)
}

/// Re-deal `owner`'s cards hidden from `viewer` among their hidden places, shuffled with `rng` (`determinize`,
/// `Game::set_unseen`).
fn redeal(g: &mut Game, viewer: usize, owner: usize, rng: &mut Rng) -> R {
    let (hand_hidden, _) = hidden_zones(viewer, owner);
    let pl = &g.st.players[owner];
    let mut deck: Vec<CardId> = pl.deck.iter().collect();
    let mut prizes: Vec<Vec<CardId>> = (0..pl.prize_count as usize).map(|i| pl.prizes[i].iter().collect()).collect();
    let mut hand: Vec<CardId> = pl.hand.iter().collect();
    let (places, mut pool) = hidden_places(g, viewer, owner);
    for i in (1..pool.len()).rev() {
        pool.swap(i, rng.index(i + 1));
    }
    for (at, c) in places.into_iter().zip(pool) {
        match at {
            Unseen::Deck(i) => deck[i] = c,
            Unseen::Prize(i, k) => prizes[i][k] = c,
            Unseen::Hand(i) => hand[i] = c,
        }
    }
    set_hidden(g, viewer, owner, &deck, &prizes, if hand_hidden { Some(&hand) } else { None })
}

impl Game {
    /// Give the cards of `owner` that `viewer` cannot see the identities `cards` (a multiset of card definitions, as many
    /// as there are such cards), dealt among their places at random with `seed`: what a search samples when the viewer
    /// doesn't know the other player's deck list (or, for their own cards, how the list splits between the deck and the
    /// Prize cards). The hidden places are [`Game::swap_hidden`]'s (`unseen_at`: the deck, unless an open prompt lists it;
    /// the hand, when it isn't the viewer's; face-down Prize cards no effect made public); every other card keeps its
    /// identity and place, every zone its size. No event; the chance stream is untouched.
    ///
    /// A hidden card is a card id whose definition is rewritten. A hidden card the state names and reads the identity of
    /// at a later decision ([`Game::named_cards`]: a Trainer card played this turn and shuffled back, the source of a
    /// lasting effect, the card of the last attack) keeps its definition (the viewer saw it), and `cards` must list it.
    /// The per-card indexes built from the definitions are rebuilt ([`Game::index_cards`]); the derived facts, the
    /// readers' card lists and the lock sync follow the layout generation.
    ///
    /// Errors, with nothing changed: `NO_SUCH_PLAYER`; `UNKNOWN_CARD` (a definition the card table doesn't have);
    /// `UNSEEN_COUNT_MISMATCH`; `NAMED_CARD_NOT_LISTED`; `RECORD_NOT_TAKEN` (the game record holds entries not taken yet:
    /// they name card ids whose identity would change under them).
    pub fn set_unseen(&mut self, viewer: usize, owner: usize, cards: &[DefId], seed: u32) -> R {
        if viewer > 1 || owner > 1 {
            return Err(GameError("NO_SUCH_PLAYER"));
        }
        if cards.iter().any(|d| *d as usize >= crate::carddb::cards().len()) {
            return Err(GameError("UNKNOWN_CARD"));
        }
        if self.record.on() && self.record.len > 0 {
            return Err(GameError("RECORD_NOT_TAKEN"));
        }
        let hidden = self.unseen_cards(viewer, owner);
        if cards.len() != hidden.len() {
            return Err(GameError("UNSEEN_COUNT_MISMATCH"));
        }
        let named = self.named_cards();
        let mut pool: Vec<DefId> = cards.to_vec();
        let mut free: Vec<CardId> = Vec::with_capacity(hidden.len());
        for c in hidden {
            if named >> c & 1 == 0 {
                free.push(c);
                continue;
            }
            let d = self.st.cards[c as usize].def;
            let i = pool.iter().position(|x| *x == d).ok_or(GameError("NAMED_CARD_NOT_LISTED"))?;
            pool.swap_remove(i);
        }
        for (c, d) in free.into_iter().zip(pool) {
            let inst = &mut self.st.cards[c as usize];
            inst.def = d;
            // A card out of play holds no Ability lock (`passive::lock_sync`).
            inst.lock_stamp = 0;
        }
        redeal(self, viewer, owner, &mut Rng::new(seed))?;
        self.index_cards();
        self.st.bump_layout();
        Ok(())
    }

    /// The cards of `owner` that `viewer` cannot see, in a fixed order (deck, Prize cards, hand): the cards
    /// [`Game::set_unseen`] gives identities to (as many as it takes) and [`determinize`] re-deals.
    pub fn unseen_cards(&self, viewer: usize, owner: usize) -> Vec<CardId> {
        if viewer > 1 || owner > 1 {
            return vec![];
        }
        hidden_places(self, viewer, owner).1
    }

    /// The cards (a bit per card id) the state names by id and reads the identity of at a later decision: the Trainer
    /// cards played this turn (`Player::played_this_turn`), the source of each lasting effect and the cards its attack
    /// and cause name, the last attack's card (`State::last_attack`, `State::player_last_attack`), the attack in progress
    /// (`Game::last_attack`) and the cards of a copied attack (`Game::copy_sessions`, `Game::deleg`). [`Game::set_unseen`]
    /// keeps their identities.
    pub fn named_cards(&self) -> u128 {
        let mut m = 0u128;
        let mut name = |c: CardId| {
            if (c as usize) < MAX_CARDS {
                m |= 1u128 << c;
            }
        };
        for pl in &self.st.players {
            pl.played_this_turn.iter().for_each(|c| name(*c));
        }
        for l in self.st.lasting.iter() {
            name(l.source);
            match l.what {
                LastingWhat::Lock { attack: Some(a), .. } | LastingWhat::Retaliate { attack: a, .. } | LastingWhat::DiscardAttackerEnergy { attack: a, .. } | LastingWhat::NextTurnBonus { attack: a, .. } => name(a.card),
                LastingWhat::Spawn { cause, .. } => Cause::unpack(cause).card.into_iter().for_each(&mut name),
                _ => {}
            }
        }
        if let Some(a) = self.st.last_attack {
            name(a.card);
        }
        for (a, src) in self.st.player_last_attack.iter().flatten() {
            name(a.card);
            name(*src);
        }
        if let Some(la) = &self.last_attack {
            name(la.attack.card);
            la.pokemon.into_iter().for_each(&mut name);
        }
        for s in self.copy_sessions.iter() {
            name(s.copycat);
            name(s.source);
        }
        if let Some(d) = self.deleg {
            name(d.copycat);
            name(d.source);
        }
        m
    }
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

    /// A face-up Prize card and a Prize card an effect made public stay where they are; the hidden cards are
    /// re-dealt among the hidden places and keep their multiset.
    #[test]
    fn determinize_keeps_visible_cards() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN]}, "opp": {"reset": true, "active": MON, "hand": [POFFIN, MON]}}));
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        g.st.players[o].prize_face_up[0] = true;
        g.st.players[o].prize_public[2] = true;
        let kept: Vec<(usize, CardId)> = [0usize, 2].iter().map(|i| (*i, g.st.players[o].prizes[*i].as_slice()[0])).collect();
        let hidden = |g: &Game| {
            let pl = &g.st.players[o];
            let mut h: Vec<CardId> = pl.deck.iter().chain(pl.hand.iter()).collect();
            for i in [1usize, 3, 4, 5] {
                h.extend(pl.prizes[i].iter());
            }
            h
        };
        let mut before = hidden(&g);
        before.sort();
        let mut moved = false;
        for seed in 1..20 {
            let old = hidden(&g);
            determinize(&mut g, v, seed).unwrap();
            for (i, c) in &kept {
                assert_eq!(g.st.players[o].prizes[*i].as_slice()[0], *c);
                assert_eq!(g.locate_card(*c).unwrap().zone, Zone::Prizes);
            }
            let mut after = hidden(&g);
            moved |= after != old;
            after.sort();
            assert_eq!(after, before);
        }
        assert!(moved, "hidden cards are still re-dealt");
    }

    fn def_of(name: &str) -> DefId {
        crate::carddb::def_by_full_name(name).unwrap()
    }

    /// The identities of `cards`, sorted (a multiset).
    fn defs(g: &Game, cards: &[CardId]) -> Vec<DefId> {
        let mut d: Vec<DefId> = cards.iter().map(|c| g.st.cards[*c as usize].def).collect();
        d.sort();
        d
    }

    /// Every card of player `p` outside `hidden`, with its place and identity.
    fn seen(g: &Game, p: usize, hidden: &[CardId]) -> Vec<(CardId, Option<crate::place::CardPlace>, DefId)> {
        (0..g.st.n_cards).filter(|c| g.st.owner(*c) == p && !hidden.contains(c)).map(|c| (c, g.locate_card(c), g.st.cards[c as usize].def)).collect()
    }

    fn table() -> Game {
        game_of(
            &[(MON, 4), (POFFIN, 4)],
            json!({"me": {"reset": true, "active": MON, "hand": [POFFIN], "discard": [MON]}, "opp": {"reset": true, "active": MON, "bench": [{"card": MON}], "hand": [POFFIN, MON], "discard": [POFFIN]}}),
        )
    }

    /// The other player's hidden cards (deck, hand, face-down Prize cards no effect made public) take the listed
    /// identities: the same card ids in the same places, the listed multiset, every zone's size kept, every card either
    /// player can see untouched (in play, discard pile, a face-up and a public Prize card, the viewer's own cards), the
    /// locations found, the per-card indexes those of the new identities, the invariants kept; the same seed gives the same
    /// game, another seed another deal.
    #[test]
    fn set_unseen_gives_hidden_cards_the_listed_identities() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        g.st.players[o].prize_face_up[0] = true;
        g.st.players[o].prize_public[2] = true;
        let sizes = |g: &Game| {
            let pl = &g.st.players[o];
            (pl.deck.len(), pl.hand.len(), (0..6).map(|i| pl.prizes[i].len()).collect::<Vec<_>>())
        };
        let size0 = sizes(&g);
        let hidden = g.unseen_cards(v, o);
        assert_eq!(hidden.len(), size0.0 + size0.1 + 4);
        let (seen_o, seen_v) = (seen(&g, o, &hidden), seen(&g, v, &[]));
        let names = ["Duraludon PRE 69", "Hero's Cape TEF 152", "Archaludon ex SSP 130", "Metal Energy MEE 8"];
        let list: Vec<DefId> = (0..hidden.len()).map(|i| def_of(names[i % 4])).collect();
        let mut again = g.fork();
        let mut other = g.fork();
        let (decls0, id0) = (g.decls.sets, g.decls.id);
        g.set_unseen(v, o, &list, 11).unwrap();
        let now = g.unseen_cards(v, o);
        let (mut a, mut b) = (now.clone(), hidden.clone());
        a.sort();
        b.sort();
        assert_eq!(a, b, "the same cards are hidden");
        let mut want = list.clone();
        want.sort();
        assert_eq!(defs(&g, &now), want);
        assert_eq!(sizes(&g), size0);
        assert_eq!(seen(&g, o, &hidden), seen_o);
        assert_eq!(seen(&g, v, &[]), seen_v);
        for (i, c) in g.st.players[o].deck.iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap(), crate::place::CardPlace { owner: o as u8, zone: Zone::Deck, index: i as u8 });
        }
        for (i, c) in g.st.players[o].hand.iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap(), crate::place::CardPlace { owner: o as u8, zone: Zone::Hand, index: i as u8 });
        }
        // The indexes are the new identities' (Hero's Cape declares an HP modifier the old cards don't).
        let fresh = crate::decl::DeclIndex::build(&g);
        assert_eq!((g.decls.sets, g.decls.kinds, g.decls.attack_passives), (fresh.sets, fresh.kinds, fresh.attack_passives));
        assert_ne!(g.decls.sets, decls0);
        assert_ne!(g.decls.id, id0);
        let mut kinds = crate::effects::KindMask::EMPTY;
        let mut locks = [0u64; 4];
        for c in 0..g.st.n_cards {
            let d = g.st.cards[c as usize].def;
            if let Some(imp) = crate::cards::impl_for(d) {
                kinds |= imp.mask;
            }
            if crate::cards::spec_for(d).map_or(false, |s| crate::spec::passive::declares_ability_lock(s.passives)) {
                locks[(c >> 6) as usize] |= 1 << (c & 63);
            }
        }
        assert_eq!((g.kinds_present.0, g.lock_cards), (kinds.0, locks));
        crate::invariants::check(&g).unwrap();
        again.set_unseen(v, o, &list, 11).unwrap();
        assert_eq!(again.state_hash(), g.state_hash());
        assert_eq!(again.st.players[o].deck.as_slice(), g.st.players[o].deck.as_slice());
        other.set_unseen(v, o, &list, 12).unwrap();
        let order = |g: &Game| g.unseen_cards(v, o).iter().map(|c| g.st.cards[*c as usize].def).collect::<Vec<_>>();
        assert_ne!(order(&other), order(&g), "another seed deals otherwise");
    }

    /// The viewer's own unseen cards (deck and face-down Prize cards; the hand is seen) take a list: the hand keeps its
    /// cards and identities.
    #[test]
    fn set_unseen_for_the_viewer_keeps_the_hand() {
        let mut g = table();
        let v = g.st.active_player as usize;
        let hand: Vec<(CardId, DefId)> = g.st.players[v].hand.iter().map(|c| (c, g.st.cards[c as usize].def)).collect();
        let hidden = g.unseen_cards(v, v);
        assert_eq!(hidden.len(), g.st.players[v].deck.len() + 6);
        assert!(hidden.iter().all(|c| !g.st.players[v].hand.contains(*c)));
        let list: Vec<DefId> = hidden.iter().map(|_| def_of(POFFIN)).collect();
        g.set_unseen(v, v, &list, 3).unwrap();
        assert_eq!(g.st.players[v].hand.iter().map(|c| (c, g.st.cards[c as usize].def)).collect::<Vec<_>>(), hand);
        assert!(g.unseen_cards(v, v).iter().all(|c| g.st.cards[*c as usize].def == def_of(POFFIN)));
        crate::invariants::check(&g).unwrap();
    }

    /// A wrong count, a definition the table doesn't have and a seat that doesn't exist are refused with nothing changed;
    /// a Trainer card played this turn and shuffled back into the deck keeps its identity, so the list must hold it.
    #[test]
    fn set_unseen_refuses_and_keeps_named_cards() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let back = g.st.players[o].deck.iter().find(|c| g.st.cdef(*c).full_name == POFFIN).unwrap();
        g.st.players[o].played_this_turn.push(back);
        let hidden = g.unseen_cards(v, o);
        let list: Vec<DefId> = hidden.iter().map(|_| def_of(MON)).collect();
        let (hash, cards) = (g.state_hash(), g.st.cards.map(|c| c.def));
        let mut unknown = list.clone();
        unknown[0] = crate::carddb::cards().len() as DefId;
        for (r, want) in [
            (g.set_unseen(v, o, &list[1..], 1), "UNSEEN_COUNT_MISMATCH"),
            (g.set_unseen(v, o, &unknown, 1), "UNKNOWN_CARD"),
            (g.set_unseen(2, o, &list, 1), "NO_SUCH_PLAYER"),
            (g.set_unseen(v, 2, &list, 1), "NO_SUCH_PLAYER"),
            (g.set_unseen(v, o, &list, 1), "NAMED_CARD_NOT_LISTED"),
        ] {
            assert_eq!(r, Err(GameError(want)));
            assert_eq!((g.state_hash(), g.st.cards.map(|c| c.def)), (hash.clone(), cards));
        }
        let mut listed = list.clone();
        listed[5] = def_of(POFFIN);
        g.set_unseen(v, o, &listed, 1).unwrap();
        assert_eq!(g.st.cdef(back).full_name, POFFIN);
        assert!(g.unseen_cards(v, o).iter().filter(|c| **c != back).all(|c| g.st.cdef(*c).full_name == MON));
        assert!(crate::engine::play_trainer::played_this_turn(&g, o, |d| d.full_name == POFFIN));
    }
}
