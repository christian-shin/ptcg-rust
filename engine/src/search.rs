//! Search support: determinization of hidden information.
//!
//! From one player's point of view the hidden cards are their own deck order
//! and face-down Prize cards, and the opponent's hand, deck, Prize cards and,
//! during setup, the Pokémon the opponent put down face down. A determinization
//! re-deals those cards among those positions, keeping every zone's size (a
//! face-down spot takes only a card that may be put down there at setup), so a
//! search can roll out a fully specified game. When the viewer doesn't know
//! which cards those are (the other player's deck list), `Game::set_unseen`
//! gives them identities from a card multiset first; when the hand and deck sizes it follows are wrong,
//! `Game::resize_unseen` moves hidden cards between the two.

use crate::carddb::{CardDef, DefId};
use crate::cause::Cause;
use crate::game::{Game, GameError, R};
use crate::list::*;
use crate::rng::Rng;
use crate::state::{LastingWhat, ListRef, SlotId, MAX_CARDS};

/// Where a card hidden from a viewer lies among its owner's cards: a position of the hand or the deck, a face-down Prize
/// card (pile, position), or a spot where it lies face down (spot, position among the spot's cards).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Unseen {
    Hand(usize),
    Deck(usize),
    Prize(usize, usize),
    FaceDown(SlotId, usize),
}

/// The other player's hand is hidden from a viewer; their own is not.
fn hand_hidden(viewer: usize, owner: usize) -> bool {
    viewer != owner
}

/// The other player's cards lying face down in play are hidden from a viewer (`State::in_play_face_down`); a player
/// knows their own.
fn face_down_hidden(g: &Game, viewer: usize, owner: usize) -> bool {
    viewer != owner && g.st.in_play_face_down()
}

/// Whether a card of definition `d` may lie at `at` among `owner`'s cards: any card in a hand, a deck or a Prize card; in
/// a face-down spot only a card that may be put down there at setup (the Active Spot: `setup::may_start_active`; a Bench
/// spot: `setup::may_bench_at_setup`).
fn fits(g: &Game, owner: usize, at: Unseen, d: &CardDef) -> bool {
    match at {
        Unseen::FaceDown(s, _) if s == g.st.players[owner].active => crate::engine::setup::may_start_active(d),
        Unseen::FaceDown(..) => crate::engine::setup::may_bench_at_setup(d),
        Unseen::Hand(_) | Unseen::Deck(_) | Unseen::Prize(..) => true,
    }
}

/// Put card `c` at `at` among `owner`'s cards, through the list's write guard (the caller bumps the layout). A face-down
/// spot's guard also stamps the card's arrival there, and the spot's top Pokémon follows the card's identity.
fn put_unseen(g: &mut Game, owner: usize, at: Unseen, c: CardId) {
    let pu = owner as u8;
    match at {
        Unseen::Hand(i) => g.st.zone_edit(ListRef::Hand(pu)).replace_at(i, c),
        Unseen::Deck(i) => g.st.zone_edit(ListRef::Deck(pu)).replace_at(i, c),
        Unseen::Prize(i, k) => g.st.zone_edit(ListRef::Prize(pu, i as u8)).replace_at(k, c),
        Unseen::FaceDown(s, k) => {
            let mut cards: Vec<CardId> = g.st.slot(owner, s).cards().iter().collect();
            cards[k] = c;
            g.st.slot_edit(owner, s).cards().set_from(&cards);
        }
    }
}

/// Whether each of `places` places can take a distinct one of `items` items it fits (`fit(place, item)`): a bipartite
/// matching by augmenting paths, so an assignment is found whenever one exists (Hall's condition), not only when a
/// greedy pass happens on it.
fn fillable(places: usize, items: usize, fit: &dyn Fn(usize, usize) -> bool) -> bool {
    fn augment(p: usize, fit: &dyn Fn(usize, usize) -> bool, seen: &mut [bool], holder: &mut [usize]) -> bool {
        for i in 0..holder.len() {
            if seen[i] || !fit(p, i) {
                continue;
            }
            seen[i] = true;
            if holder[i] == usize::MAX || augment(holder[i], fit, seen, holder) {
                holder[i] = p;
                return true;
            }
        }
        false
    }
    let mut holder = vec![usize::MAX; items];
    (0..places).all(|p| augment(p, fit, &mut vec![false; items], &mut holder))
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

    /// Swap two cards of one owner that `viewer` cannot see, under the rule [`determinize`] re-deals by (`unseen_at`):
    /// the other player's hand, either deck, face-down Prize cards no effect made public, no open prompt listing the
    /// deck, and during setup the other player's Pokémon put down face down. Each takes the other's place; no event, no
    /// shuffle, every zone keeps its size, the place records and the layout generation follow. Refused with
    /// `CARD_NOT_HIDDEN` otherwise (a card in play face up, in the viewer's own hand, in a discard pile or the Lost
    /// Zone, set aside, face up; cards of two players), and with `FACE_DOWN_SPOT_REFUSES_CARD` when a card would go to
    /// a face-down spot it may not be put down in at setup. A host that plays one seat holds its guesses as a
    /// determinization and re-identifies one when the real card is revealed.
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
        if !fits(self, owner, pa, self.st.cdef(b)) || !fits(self, owner, pb, self.st.cdef(a)) {
            return Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD"));
        }
        put_unseen(self, owner, pa, b);
        put_unseen(self, owner, pb, a);
        self.st.bump_layout();
        Ok(())
    }

    /// The place of card `c` when no player can see it ([`Game::swap_unseen`]).
    /// With a `viewer`, hidden from that viewer: also the owner's hand (`hand_hidden`) and the owner's cards lying face
    /// down in play (`face_down_hidden`) when the owner is the other player.
    fn unseen_at(&self, c: CardId, viewer: Option<usize>) -> Option<Unseen> {
        if c >= self.st.n_cards {
            return None;
        }
        let p = self.st.owner(c);
        let pl = &self.st.players[p];
        let hand_hidden = viewer.map_or(false, |v| hand_hidden(v, p));
        let face_down_hidden = viewer.map_or(false, |v| face_down_hidden(self, v, p));
        let at = match self.st.locate(c)? {
            ListRef::Hand(q) if q as usize == p && hand_hidden => Unseen::Hand(pl.hand().index_of(c)?),
            ListRef::Slot(q, s) if q as usize == p && face_down_hidden => Unseen::FaceDown(s, pl.slots[s as usize].cards().index_of(c)?),
            ListRef::Deck(q) if q as usize == p => Unseen::Deck(pl.deck().index_of(c)?),
            ListRef::Prize(q, i) if q as usize == p && !pl.prize_face_up[i as usize] && !pl.prize_public[i as usize] => Unseen::Prize(i as usize, pl.prizes()[i as usize].index_of(c)?),
            _ => return None,
        };
        if matches!(at, Unseen::Deck(_)) && self.deck_shown(p) {
            return None;
        }
        Some(at)
    }

    /// An open prompt lists `owner`'s deck (a search shows the deck it searches): its cards are seen.
    fn deck_shown(&self, owner: usize) -> bool {
        let shown = |r: ListRef| r == ListRef::Deck(owner as u8);
        self.prompts.iter().filter(|pr| !pr.answered).any(|pr| match pr.kind {
            crate::prompts::PromptKind::ChooseCards { cards, .. } | crate::prompts::PromptKind::OrderCards { cards, .. } | crate::prompts::PromptKind::AttachEnergy { cards, .. } => shown(cards),
            _ => false,
        })
    }
}

/// Uniformly random determinization for `viewer`, and a fresh chance seed. Only cards the viewer cannot see move
/// (`Game::unseen_at` with the viewer, the rule `Game::swap_hidden` uses): a face-up or public Prize card, a deck an
/// open prompt lists and every card outside hand, deck, Prize cards and face-down spots stay where they are; the hidden
/// cards of each owner are re-dealt among the hidden places, keeping every zone's size, a face-down spot taking only a
/// card that may be put down there at setup (`deal`).
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    let mut rng = Rng::new(seed);
    for owner in 0..2 {
        redeal(g, viewer, owner, &mut rng)?;
    }
    g.rng = Rng::new(rng.next_u32());
    Ok(())
}

/// The places of `owner`'s cards hidden from `viewer` (`Game::unseen_at`), in a fixed order (deck, Prize cards, hand,
/// face-down spots: the Active Spot, then the Bench), and the cards in them.
fn hidden_places(g: &Game, viewer: usize, owner: usize) -> (Vec<Unseen>, Vec<CardId>) {
    let pl = &g.st.players[owner];
    let mut places: Vec<Unseen> = vec![];
    let mut cards: Vec<CardId> = vec![];
    let spots: Vec<(CardId, Unseen)> = pl.in_play().flat_map(|s| pl.slots[s as usize].cards().iter().enumerate().map(move |(k, c)| (c, Unseen::FaceDown(s, k))).collect::<Vec<_>>()).collect();
    let all = pl.deck().iter().enumerate().map(|(i, c)| (c, Unseen::Deck(i)))
        .chain((0..pl.prize_count as usize).flat_map(|i| pl.prizes()[i].iter().enumerate().map(move |(k, c)| (c, Unseen::Prize(i, k)))))
        .chain(pl.hand().iter().enumerate().map(|(i, c)| (c, Unseen::Hand(i))))
        .chain(spots);
    for (c, at) in all {
        if g.unseen_at(c, Some(viewer)) == Some(at) {
            places.push(at);
            cards.push(c);
        }
    }
    (places, cards)
}

/// Re-deal `owner`'s cards hidden from `viewer` among their hidden places, shuffled with `rng` (`determinize`): every
/// hidden card lands at a uniformly random hidden place that may hold it (`deal`).
fn redeal(g: &mut Game, viewer: usize, owner: usize, rng: &mut Rng) -> R {
    let (places, mut pool) = hidden_places(g, viewer, owner);
    shuffle(&mut pool, rng);
    let dealt = deal(g, owner, &places, pool, |c| g.st.cdef(c), rng)?;
    for (at, c) in places.into_iter().zip(dealt) {
        put_unseen(g, owner, at, c);
    }
    g.st.bump_layout();
    Ok(())
}

/// Shuffle `v` with `rng` (Fisher-Yates, last position first).
fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        v.swap(i, rng.index(i + 1));
    }
}

/// The shuffled `pool` dealt to `places` (one item each, in the order of `places`; `def` says what card an item is). With
/// no face-down spot among them the pool is the deal (and no chance is drawn). Otherwise each face-down spot, the one
/// fewest items fit first, takes an item drawn with `rng` uniformly among the items that fit it (`fits`) and leave the
/// other spots fillable (`fillable`); the other places take the remaining items in their shuffled order. The sets of
/// cards that fit the Active Spot and a Bench spot are nested (a card that may be Benched at setup may start as the Active
/// Pokémon), so every deal that respects the spots is equally likely. `FACE_DOWN_SPOT_UNFILLABLE` when no deal respects
/// them.
fn deal<T: Copy>(g: &Game, owner: usize, places: &[Unseen], pool: Vec<T>, def: impl Fn(T) -> &'static CardDef, rng: &mut Rng) -> R<Vec<T>> {
    let face_down = |k: &usize| matches!(places[*k], Unseen::FaceDown(..));
    let mut spots: Vec<usize> = (0..places.len()).filter(face_down).collect();
    if spots.is_empty() {
        return Ok(pool);
    }
    let fit = |k: usize, x: T| fits(g, owner, places[k], def(x));
    spots.sort_by_key(|k| pool.iter().filter(|x| fit(*k, **x)).count());
    let mut left = pool;
    let mut out: Vec<Option<T>> = vec![None; places.len()];
    for j in 0..spots.len() {
        let (k, rest) = (spots[j], &spots[j + 1..]);
        let ok: Vec<usize> = (0..left.len()).filter(|&i| fit(k, left[i]) && fillable(rest.len(), left.len(), &|a, b| b != i && fit(rest[a], left[b]))).collect();
        if ok.is_empty() {
            return Err(GameError("FACE_DOWN_SPOT_UNFILLABLE"));
        }
        out[k] = Some(left.remove(ok[rng.index(ok.len())]));
    }
    let mut rest = left.into_iter();
    for k in (0..places.len()).filter(|k| !face_down(k)) {
        out[k] = rest.next();
    }
    Ok(out.into_iter().map(|x| x.expect("as many items as places")).collect())
}

/// What a hidden place gets in [`Game::set_unseen`]'s plan: a card that keeps its identity ([`Game::named_cards`]), or an
/// identity from the list.
#[derive(Clone, Copy)]
enum Planned {
    Kept(CardId),
    Def(DefId),
}

impl Game {
    /// The checks every identity change of hidden cards starts with: the seats exist, the definitions are in the card
    /// table, the game record holds no entry not taken yet (it names card ids whose identity would change under them).
    fn unseen_change_allowed(&self, viewer: usize, owner: usize, defs: impl Iterator<Item = DefId>) -> R {
        if viewer > 1 || owner > 1 {
            return Err(GameError("NO_SUCH_PLAYER"));
        }
        let n = crate::carddb::cards().len();
        if defs.into_iter().any(|d| d as usize >= n) {
            return Err(GameError("UNKNOWN_CARD"));
        }
        if self.record.on() && self.record.len > 0 {
            return Err(GameError("RECORD_NOT_TAKEN"));
        }
        Ok(())
    }

    /// Give the cards of `owner` that `viewer` cannot see the identities `cards` (a multiset of card definitions, as many
    /// as there are such cards), dealt among their places at random with `seed`: what a search samples when the viewer
    /// doesn't know the other player's deck list (or, for their own cards, how the list splits between the deck and the
    /// Prize cards). The hidden places are [`Game::swap_hidden`]'s (`unseen_at`: the deck, unless an open prompt lists it;
    /// the hand, when it isn't the viewer's; face-down Prize cards no effect made public; during setup the Pokémon the
    /// other player put down face down); every other card keeps its identity and place, every zone its size. A
    /// face-down spot gets only a card that may be put down there at setup (a Basic Pokémon; on the Active Spot also a
    /// card that may start as the Active Pokémon), drawn uniformly among the deals that respect the spots. No event;
    /// the chance stream is untouched.
    ///
    /// A hidden card the state names and reads the identity of at a later decision ([`Game::named_cards`]: a Trainer card
    /// played this turn and shuffled back, the source of a lasting effect, the card of the last attack) keeps its
    /// definition (the viewer saw it), and `cards` must list it; it lands at a random hidden place like the others.
    ///
    /// The plan (a deal of the kept cards and the listed identities over the hidden places) is made first; then the kept
    /// cards go to their places, and [`Game::set_unseen_assigned`] gives the other hidden cards, in place, their planned
    /// identities (the one write path of identities).
    ///
    /// Errors, with nothing changed: `NO_SUCH_PLAYER`; `UNKNOWN_CARD` (a definition the card table doesn't have);
    /// `RECORD_NOT_TAKEN` (the game record holds entries not taken yet); `UNSEEN_COUNT_MISMATCH`;
    /// `NAMED_CARD_NOT_LISTED`; `FACE_DOWN_SPOT_UNFILLABLE` (no deal of `cards` gives every face-down spot a card that
    /// may be put down there).
    pub fn set_unseen(&mut self, viewer: usize, owner: usize, cards: &[DefId], seed: u32) -> R {
        self.unseen_change_allowed(viewer, owner, cards.iter().copied())?;
        let (places, hidden) = hidden_places(self, viewer, owner);
        if cards.len() != hidden.len() {
            return Err(GameError("UNSEEN_COUNT_MISMATCH"));
        }
        let named = self.named_cards();
        let kept = |c: CardId| named >> c & 1 != 0;
        let mut pool: Vec<Planned> = cards.iter().map(|d| Planned::Def(*d)).collect();
        for &c in hidden.iter().filter(|c| kept(**c)) {
            let d = self.st.cards[c as usize].def;
            let i = pool.iter().position(|x| matches!(x, Planned::Def(y) if *y == d)).ok_or(GameError("NAMED_CARD_NOT_LISTED"))?;
            pool[i] = Planned::Kept(c);
        }
        let mut rng = Rng::new(seed);
        shuffle(&mut pool, &mut rng);
        let plan = deal(self, owner, &places, pool, |x| match x {
            Planned::Kept(c) => self.st.cdef(c),
            Planned::Def(d) => crate::carddb::def(d),
        }, &mut rng)?;
        // The kept cards go to their planned places; the other hidden cards fill the rest, in their order, and take the
        // planned identities there.
        let mut others = hidden.iter().copied().filter(|c| !kept(*c));
        let mut assigned: Vec<(CardId, DefId)> = Vec::with_capacity(hidden.len());
        for (at, x) in places.iter().zip(plan) {
            let c = match x {
                Planned::Kept(c) => c,
                Planned::Def(d) => {
                    let c = others.next().expect("as many cards as places");
                    assigned.push((c, d));
                    c
                }
            };
            put_unseen(self, owner, *at, c);
        }
        self.st.bump_layout();
        self.set_unseen_assigned(viewer, owner, &assigned)
    }

    /// Give each listed card of `owner` that `viewer` cannot see the listed identity, in place: what a host that names
    /// the hidden cards by its own ids does when it decides which card is which. Nothing moves; no event; the chance
    /// stream is untouched. The list may cover all of [`Game::unseen_cards`] or a part: an unlisted hidden card keeps
    /// its identity. A card [`Game::named_cards`] names keeps its identity (it may be listed with that identity). During
    /// setup a face-down spot must end holding a card that may be put down there (`setup::may_start_active` for the
    /// Active Spot, `setup::may_bench_at_setup` for the Bench). The per-card indexes are rebuilt
    /// ([`Game::index_cards`]); the derived facts, the readers' card lists and the lock sync follow the layout generation.
    ///
    /// Errors, with nothing changed: `NO_SUCH_PLAYER`; `UNKNOWN_CARD`; `RECORD_NOT_TAKEN`; `CARD_NOT_HIDDEN` (a card
    /// that isn't one of the owner's cards hidden from the viewer); `DUPLICATE_CARD` (a card listed twice);
    /// `NAMED_CARD_KEEPS_IDENTITY` (a named card listed with another identity); `FACE_DOWN_SPOT_REFUSES_CARD`.
    pub fn set_unseen_assigned(&mut self, viewer: usize, owner: usize, assigned: &[(CardId, DefId)]) -> R {
        self.unseen_change_allowed(viewer, owner, assigned.iter().map(|x| x.1))?;
        let (places, hidden) = hidden_places(self, viewer, owner);
        let named = self.named_cards();
        let mut listed = 0u128;
        for &(c, d) in assigned {
            if !hidden.contains(&c) {
                return Err(GameError("CARD_NOT_HIDDEN"));
            }
            if listed >> c & 1 != 0 {
                return Err(GameError("DUPLICATE_CARD"));
            }
            listed |= 1u128 << c;
            if named >> c & 1 != 0 && self.st.cards[c as usize].def != d {
                return Err(GameError("NAMED_CARD_KEEPS_IDENTITY"));
            }
        }
        let new_def = |c: CardId| assigned.iter().find(|x| x.0 == c).map_or(self.st.cards[c as usize].def, |x| x.1);
        for (at, c) in places.iter().zip(&hidden) {
            if matches!(at, Unseen::FaceDown(..)) && !fits(self, owner, *at, crate::carddb::def(new_def(*c))) {
                return Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD"));
            }
        }
        for &(c, d) in assigned {
            if self.st.cards[c as usize].def != d {
                self.st.cards[c as usize].def = d;
                // A card out of play holds no Ability lock (`passive::lock_sync`); a face-down card in play takes hold
                // anew at the next lock sync, as the card it now is.
                self.st.set_lock_stamp(c, 0);
            }
        }
        // A face-down spot's top Pokémon follows its card's identity (recomputed when the spot's guard drops).
        for at in places {
            if let Unseen::FaceDown(s, _) = at {
                drop(self.st.slot_edit(owner, s));
            }
        }
        self.index_cards();
        self.st.bump_layout();
        Ok(())
    }

    /// Move cards of `owner` that `viewer` cannot see between the owner's hand and deck so that they hold `hand` and
    /// `deck` cards: what a host does when the public counts of the real game (the hand size, the deck size) differ from
    /// the game it follows because it could not follow a hidden move (a card from the other player's hand into their
    /// deck, a search it inferred wrongly). The cards that move are drawn with `seed` among those of the shrinking zone
    /// the state doesn't name ([`Game::named_cards`]: those stay where they are); a card put into the deck goes to a
    /// random position (the deck order is hidden from both players), a card put into the hand after the others. Nothing
    /// else moves: the cards the viewer sees, the Prize cards, the face-down Pokémon of setup. No event; identities are
    /// kept; the chance stream is untouched. The place records and the layout generation follow.
    ///
    /// The Prize cards are not re-sized: their count changes only by public actions (Prize cards taken; turning one face
    /// up), which a following host sees, and every effect of the pool that moves cards into or out of them keeps their
    /// count (e.g. Redeemable Ticket). A different count of face-down Prize cards is a different public history (the
    /// Prize cards taken), not a hidden move.
    ///
    /// Errors, with nothing changed: `NO_SUCH_PLAYER`; `UNSEEN_COUNT_MISMATCH` (`hand + deck` isn't the owner's hand and
    /// deck sizes now: the total is kept); `ZONE_NOT_HIDDEN` (the size of a zone the viewer sees would change: the
    /// viewer's own hand, a deck an open prompt lists); `NAMED_CARD_KEEPS_PLACE` (the shrinking zone has fewer cards the
    /// state doesn't name than must leave it).
    pub fn resize_unseen(&mut self, viewer: usize, owner: usize, hand: usize, deck: usize, seed: u32) -> R {
        if viewer > 1 || owner > 1 {
            return Err(GameError("NO_SUCH_PLAYER"));
        }
        let pl = &self.st.players[owner];
        let (hand_now, deck_now) = (pl.hand().len(), pl.deck().len());
        if hand + deck != hand_now + deck_now {
            return Err(GameError("UNSEEN_COUNT_MISMATCH"));
        }
        if hand == hand_now {
            return Ok(());
        }
        if !hand_hidden(viewer, owner) || self.deck_shown(owner) {
            return Err(GameError("ZONE_NOT_HIDDEN"));
        }
        let pu = owner as u8;
        let (from, to) = if hand > hand_now { (ListRef::Deck(pu), ListRef::Hand(pu)) } else { (ListRef::Hand(pu), ListRef::Deck(pu)) };
        let named = self.named_cards();
        let mut moving: Vec<CardId> = self.st.list(from).as_slice().iter().copied().filter(|c| named >> c & 1 == 0).collect();
        let n = hand.abs_diff(hand_now);
        if moving.len() < n {
            return Err(GameError("NAMED_CARD_KEEPS_PLACE"));
        }
        let mut rng = Rng::new(seed);
        shuffle(&mut moving, &mut rng);
        moving.truncate(n);
        let left: Vec<CardId> = self.st.list(from).as_slice().iter().copied().filter(|c| !moving.contains(c)).collect();
        let mut grown: Vec<CardId> = self.st.list(to).as_slice().to_vec();
        for &c in &moving {
            let i = if to == ListRef::Hand(pu) { grown.len() } else { rng.index(grown.len() + 1) };
            grown.insert(i, c);
        }
        self.st.list_mut(from).set_from(&left);
        self.st.list_mut(to).set_from(&grown);
        Ok(())
    }

    /// The cards of `owner` that `viewer` cannot see, in a fixed order (deck, Prize cards, hand, face-down spots): the cards
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
pub(crate) mod tests {
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
        let deck = g.st.players[p].deck().as_slice().to_vec();
        let (a, b) = (deck[0], deck[5]);
        g.swap_unseen(a, b).unwrap();
        assert_eq!((g.st.players[p].deck().as_slice()[0], g.st.players[p].deck().as_slice()[5]), (b, a));
        assert_eq!(g.locate_card(b).unwrap().index, 0);
        assert_eq!(g.locate_card(a).unwrap().index, 5);
        let prize = g.st.players[p].prizes()[2].as_slice()[0];
        g.swap_unseen(a, prize).unwrap();
        assert_eq!(g.st.players[p].prizes()[2].as_slice()[0], a);
        assert_eq!(g.st.players[p].deck().as_slice()[5], prize);
        assert_eq!(g.locate_card(a).unwrap().zone, Zone::Prizes);
        assert_eq!(g.locate_card(prize).unwrap().zone, Zone::Deck);
        assert_eq!(g.st.players[p].deck().len(), deck.len());
    }

    /// A card a player can see is never swapped: a hand card, a Pokémon in play, a face-up Prize card, cards of two
    /// players, and the deck while a search's prompt lists it.
    #[test]
    fn a_seen_card_is_not_swapped() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN]}, "opp": {"reset": true, "active": MON}}));
        let p = g.st.active_player as usize;
        let top = g.st.players[p].deck().as_slice()[0];
        let refused = Err(GameError("CARD_NOT_UNSEEN"));
        let hand = g.st.players[p].hand().as_slice()[0];
        assert_eq!(g.swap_unseen(top, hand), refused);
        let active = g.st.active_pokemon(p).unwrap();
        assert_eq!(g.swap_unseen(active, top), refused);
        let theirs = g.st.players[1 - p].deck().as_slice()[0];
        assert_eq!(g.swap_unseen(top, theirs), refused);
        g.st.players[p].prize_face_up[0] = true;
        let up = g.st.players[p].prizes()[0].as_slice()[0];
        assert_eq!(g.swap_unseen(top, up), refused);
        // While the search's prompt lists the deck, its cards are seen by the player searching.
        let k = g.st.players[p].hand().iter().position(|c| g.st.cdef(c).name == "Buddy-Buddy Poffin").unwrap();
        let play = crate::options::legal_actions(&g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
        g.act(play).unwrap();
        g.select().unwrap().unwrap();
        let deck = g.st.players[p].deck().as_slice().to_vec();
        assert_eq!(g.swap_unseen(deck[0], deck[1]), refused);
        let down = g.st.players[p].prizes()[1].as_slice()[0];
        let other = g.st.players[p].prizes()[3].as_slice()[0];
        assert_eq!(g.swap_unseen(down, other), Ok(()), "face-down Prize cards stay unseen during the search");
    }

    /// A guessed opponent hand card trades places with a card of their deck; the viewer's own hand, a public card and
    /// cards of two players are refused; a determinized game stays consistent.
    #[test]
    fn hidden_cards_trade_places_for_a_viewer() {
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN]}, "opp": {"reset": true, "active": MON, "hand": [POFFIN, MON]}}));
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let refused = Err(GameError("CARD_NOT_HIDDEN"));
        let (h, d) = (g.st.players[o].hand().as_slice()[0], g.st.players[o].deck().as_slice()[3]);
        let (hn, dn) = (g.st.players[o].hand().len(), g.st.players[o].deck().len());
        let gen = g.st.layout_gen;
        g.swap_hidden(v, h, d).unwrap();
        assert_eq!((g.st.players[o].hand().as_slice()[0], g.st.players[o].deck().as_slice()[3]), (d, h));
        assert_eq!(g.locate_card(d).unwrap().zone, Zone::Hand);
        assert_eq!(g.locate_card(h).unwrap().index, 3);
        assert_eq!((g.st.players[o].hand().len(), g.st.players[o].deck().len()), (hn, dn));
        assert_ne!(g.st.layout_gen, gen);
        let pz = g.st.players[o].prizes()[1].as_slice()[0];
        g.swap_hidden(v, pz, d).unwrap();
        assert_eq!(g.locate_card(pz).unwrap().zone, Zone::Hand);
        // The viewer's own hand is seen by the viewer; it is hidden only from the other player.
        let mine = g.st.players[v].hand().as_slice()[0];
        let mine_deck = g.st.players[v].deck().as_slice()[0];
        assert_eq!(g.swap_hidden(v, mine, mine_deck), refused);
        assert_eq!(g.swap_hidden(o, mine, mine_deck), Ok(()));
        // Public cards: in play, cards of two owners, a face-up Prize card, a bad viewer.
        let active = g.st.active_pokemon(o).unwrap();
        let top = g.st.players[o].deck().as_slice()[0];
        assert_eq!(g.swap_hidden(v, active, top), refused);
        assert_eq!(g.swap_hidden(v, top, g.st.players[v].deck().as_slice()[1]), refused);
        g.st.players[o].prize_face_up[0] = true;
        let up = g.st.players[o].prizes()[0].as_slice()[0];
        assert_eq!(g.swap_hidden(v, top, up), refused);
        assert_eq!(g.swap_hidden(2, top, top), refused);
        // Consistent after determinizing and swapping: every card is found where its list holds it.
        determinize(&mut g, v, 7).unwrap();
        let (x, y) = (g.st.players[o].hand().as_slice()[0], g.st.players[o].deck().as_slice()[0]);
        g.swap_hidden(v, x, y).unwrap();
        for (i, c) in g.st.players[o].hand().as_slice().to_vec().into_iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap().index as usize, i);
        }
        for (i, c) in g.st.players[o].deck().as_slice().to_vec().into_iter().enumerate() {
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
        let kept: Vec<(usize, CardId)> = [0usize, 2].iter().map(|i| (*i, g.st.players[o].prizes()[*i].as_slice()[0])).collect();
        let hidden = |g: &Game| {
            let pl = &g.st.players[o];
            let mut h: Vec<CardId> = pl.deck().iter().chain(pl.hand().iter()).collect();
            for i in [1usize, 3, 4, 5] {
                h.extend(pl.prizes()[i].iter());
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
                assert_eq!(g.st.players[o].prizes()[*i].as_slice()[0], *c);
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
            (pl.deck().len(), pl.hand().len(), (0..6).map(|i| pl.prizes()[i].len()).collect::<Vec<_>>())
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
        for (i, c) in g.st.players[o].deck().iter().enumerate() {
            assert_eq!(g.locate_card(c).unwrap(), crate::place::CardPlace { owner: o as u8, zone: Zone::Deck, index: i as u8 });
        }
        for (i, c) in g.st.players[o].hand().iter().enumerate() {
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
        assert_eq!(again.st.players[o].deck().as_slice(), g.st.players[o].deck().as_slice());
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
        let hand: Vec<(CardId, DefId)> = g.st.players[v].hand().iter().map(|c| (c, g.st.cards[c as usize].def)).collect();
        let hidden = g.unseen_cards(v, v);
        assert_eq!(hidden.len(), g.st.players[v].deck().len() + 6);
        assert!(hidden.iter().all(|c| !g.st.players[v].hand().contains(*c)));
        let list: Vec<DefId> = hidden.iter().map(|_| def_of(POFFIN)).collect();
        g.set_unseen(v, v, &list, 3).unwrap();
        assert_eq!(g.st.players[v].hand().iter().map(|c| (c, g.st.cards[c as usize].def)).collect::<Vec<_>>(), hand);
        assert!(g.unseen_cards(v, v).iter().all(|c| g.st.cards[*c as usize].def == def_of(POFFIN)));
        crate::invariants::check(&g).unwrap();
    }

    /// A wrong count, a definition the table doesn't have and a seat that doesn't exist are refused with nothing changed;
    /// a Trainer card played this turn and shuffled back into the deck keeps its identity, so the list must hold it.
    #[test]
    fn set_unseen_refuses_and_keeps_named_cards() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let back = g.st.players[o].deck().iter().find(|c| g.st.cdef(*c).full_name == POFFIN).unwrap();
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

    /// Listed hidden cards take the listed identities in place: every card keeps its place, unlisted cards keep their
    /// identity, the per-card indexes are the new identities'; a named card may be listed with its own identity.
    #[test]
    fn set_unseen_assigned_relabels_in_place() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let back = g.st.players[o].deck().iter().find(|c| g.st.cdef(*c).full_name == POFFIN).unwrap();
        g.st.players[o].played_this_turn.push(back);
        let hidden = g.unseen_cards(v, o);
        let places = |g: &Game| (0..g.st.n_cards).map(|c| g.locate_card(c)).collect::<Vec<_>>();
        let (before, defs0) = (places(&g), g.st.cards.map(|c| c.def));
        let hand = g.st.players[o].hand().as_slice()[0];
        let free: Vec<CardId> = hidden.iter().copied().filter(|c| *c != back && *c != hand).take(2).collect();
        let assigned = [(free[0], def_of("Hero's Cape TEF 152")), (hand, def_of("Duraludon PRE 69")), (free[1], def_of("Metal Energy MEE 8")), (back, def_of(POFFIN))];
        g.set_unseen_assigned(v, o, &assigned).unwrap();
        assert_eq!(places(&g), before, "nothing moves");
        for c in 0..g.st.n_cards {
            let want = assigned.iter().find(|x| x.0 == c).map_or(defs0[c as usize], |x| x.1);
            assert_eq!(g.st.cards[c as usize].def, want);
        }
        let fresh = crate::decl::DeclIndex::build(&g);
        assert_eq!((g.decls.sets, g.decls.kinds, g.decls.attack_passives), (fresh.sets, fresh.kinds, fresh.attack_passives));
        crate::invariants::check(&g).unwrap();
        assert!(g.set_unseen_assigned(v, o, &[]).is_ok(), "an empty list changes nothing");
    }

    /// Refused lists change nothing: a card not hidden from the viewer (in play; the viewer's own hand; another owner's),
    /// a card listed twice, a named card given another identity, an unknown definition, a seat that doesn't exist.
    #[test]
    fn set_unseen_assigned_refuses_with_nothing_changed() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let back = g.st.players[o].deck().iter().find(|c| g.st.cdef(*c).full_name == POFFIN).unwrap();
        g.st.players[o].played_this_turn.push(back);
        let top = g.st.players[o].deck().iter().find(|c| *c != back).unwrap();
        let mon = def_of(MON);
        let (hash, defs) = (g.state_hash(), g.st.cards.map(|c| c.def));
        for (r, want) in [
            (g.set_unseen_assigned(v, o, &[(g.st.active_pokemon(o).unwrap(), mon)]), "CARD_NOT_HIDDEN"),
            (g.set_unseen_assigned(o, o, &[(g.st.players[o].hand().as_slice()[0], mon)]), "CARD_NOT_HIDDEN"),
            (g.set_unseen_assigned(v, o, &[(g.st.players[v].deck().as_slice()[0], mon)]), "CARD_NOT_HIDDEN"),
            (g.set_unseen_assigned(v, o, &[(top, mon), (top, mon)]), "DUPLICATE_CARD"),
            (g.set_unseen_assigned(v, o, &[(back, mon)]), "NAMED_CARD_KEEPS_IDENTITY"),
            (g.set_unseen_assigned(v, o, &[(top, crate::carddb::cards().len() as DefId)]), "UNKNOWN_CARD"),
            (g.set_unseen_assigned(2, o, &[(top, mon)]), "NO_SUCH_PLAYER"),
        ] {
            assert_eq!(r, Err(GameError(want)));
            assert_eq!((g.state_hash(), g.st.cards.map(|c| c.def)), (hash.clone(), defs));
        }
    }

    /// The other player's hand grows from their deck and shrinks back into it: the sizes asked, the same cards with the
    /// same identities, a named deck card stays in the deck, the Prize cards and every card outside hand and deck in
    /// place, the viewer's cards untouched, the locations found, the invariants kept; the same seed moves the same cards,
    /// another seed others.
    #[test]
    fn resize_unseen_moves_hidden_cards_between_hand_and_deck() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let named = g.st.players[o].deck().as_slice()[0];
        g.st.players[o].played_this_turn.push(named);
        let (h, d) = (g.st.players[o].hand().len(), g.st.players[o].deck().len());
        let zones = |g: &Game| {
            let pl = &g.st.players[o];
            let mut c: Vec<CardId> = pl.hand().iter().chain(pl.deck().iter()).collect();
            c.sort();
            c
        };
        let in_zones = zones(&g);
        let outside = |g: &Game| seen(g, o, &[]).into_iter().filter(|x| !in_zones.contains(&x.0)).collect::<Vec<_>>();
        let (outside0, mine0, defs0, prizes0) = (outside(&g), seen(&g, v, &[]), g.st.cards.map(|c| c.def), *g.st.players[o].prizes());
        let hand0: Vec<CardId> = g.st.players[o].hand().as_slice().to_vec();
        let mut again = g.fork();
        let mut other = g.fork();
        let gen = g.st.layout_gen;
        g.resize_unseen(v, o, h + 5, d - 5, 3).unwrap();
        let check = |g: &Game, hand: usize| {
            let pl = &g.st.players[o];
            assert_eq!((pl.hand().len(), pl.deck().len()), (hand, h + d - hand));
            assert_eq!(zones(g), in_zones);
            assert!(pl.deck().contains(named), "the named card stays in the deck");
            assert_eq!((outside(g), seen(g, v, &[]), g.st.cards.map(|c| c.def)), (outside0.clone(), mine0.clone(), defs0));
            assert!((0..6).all(|i| pl.prizes()[i].as_slice() == prizes0[i].as_slice()));
            for (i, c) in pl.hand().iter().enumerate() {
                assert_eq!(g.locate_card(c).unwrap(), crate::place::CardPlace { owner: o as u8, zone: Zone::Hand, index: i as u8 });
            }
            for (i, c) in pl.deck().iter().enumerate() {
                assert_eq!(g.locate_card(c).unwrap(), crate::place::CardPlace { owner: o as u8, zone: Zone::Deck, index: i as u8 });
            }
            crate::invariants::check(g).unwrap();
        };
        check(&g, h + 5);
        assert_eq!(&g.st.players[o].hand().as_slice()[..h], &hand0[..], "the hand keeps its cards, the new ones after them");
        assert_ne!(g.st.layout_gen, gen);
        again.resize_unseen(v, o, h + 5, d - 5, 3).unwrap();
        assert_eq!(again.state_hash(), g.state_hash());
        other.resize_unseen(v, o, h + 5, d - 5, 4).unwrap();
        assert_ne!(other.st.players[o].hand().as_slice(), g.st.players[o].hand().as_slice(), "another seed moves other cards");
        // Back into the deck, the whole hand; the same sizes change nothing.
        g.resize_unseen(v, o, 0, h + d, 5).unwrap();
        check(&g, 0);
        let hash = g.state_hash();
        g.resize_unseen(v, o, 0, h + d, 6).unwrap();
        assert_eq!(g.state_hash(), hash);
    }

    /// Refused sizes change nothing: a different total, a bad seat, the viewer's own hand, a deck an open prompt lists,
    /// a hand whose named cards can't leave it.
    #[test]
    fn resize_unseen_refuses_with_nothing_changed() {
        let mut g = table();
        let (v, o) = (g.st.active_player as usize, 1 - g.st.active_player as usize);
        let (h, d) = (g.st.players[o].hand().len(), g.st.players[o].deck().len());
        for c in g.st.players[o].hand().as_slice().to_vec() {
            g.st.players[o].played_this_turn.push(c);
        }
        let hash = g.state_hash();
        for (r, want) in [
            (g.resize_unseen(v, o, h + 1, d, 1), "UNSEEN_COUNT_MISMATCH"),
            (g.resize_unseen(2, o, h, d, 1), "NO_SUCH_PLAYER"),
            (g.resize_unseen(v, 2, h, d, 1), "NO_SUCH_PLAYER"),
            (g.resize_unseen(o, o, h + 1, d - 1, 1), "ZONE_NOT_HIDDEN"),
            (g.resize_unseen(v, o, h - 1, d + 1, 1), "NAMED_CARD_KEEPS_PLACE"),
        ] {
            assert_eq!(r, Err(GameError(want)));
            assert_eq!(g.state_hash(), hash);
        }
        assert_eq!(g.resize_unseen(o, o, h, d, 1), Ok(()), "the viewer's own sizes unchanged: nothing to move");
        g.resize_unseen(v, o, h + 1, d - 1, 1).unwrap();
        // While a search's prompt lists the deck, the searching player's deck is seen.
        let mut g = game_of(&[(MON, 4), (POFFIN, 4)], json!({"me": {"reset": true, "active": MON, "hand": [POFFIN, MON]}, "opp": {"reset": true, "active": MON}}));
        let p = g.st.active_player as usize;
        let k = g.st.players[p].hand().iter().position(|c| g.st.cdef(c).name == "Buddy-Buddy Poffin").unwrap();
        let play = crate::options::legal_actions(&g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
        g.act(play).unwrap();
        g.select().unwrap().unwrap();
        let (h, d) = (g.st.players[p].hand().len(), g.st.players[p].deck().len());
        let hash = g.state_hash();
        assert_eq!(g.resize_unseen(1 - p, p, h + 1, d - 1, 1), Err(GameError("ZONE_NOT_HIDDEN")));
        assert_eq!(g.state_hash(), hash);
    }

    // -----------------------------------------------------------------------------------------------------------
    // The Pokémon put down face down at setup (SSP rulebook p. 8 steps 5, 6 and 8; APR G steps 5 to 8)

    const CINDERACE: &str = "Cinderace MEG 28";

    /// Answer the open decision taking every option allowed, up to its maximum (a setup choice puts down every Basic
    /// Pokémon of the hand).
    pub(crate) fn answer_all(g: &mut Game) {
        let sel = g.select_with(false).unwrap().expect("a decision");
        let mut picks: Vec<usize> = Vec::new();
        while picks.len() < sel.max_count.max(sel.min_count) {
            let (mask, _) = g.pick_mask(&sel, &picks);
            let Some(j) = (0..mask.len()).find(|j| mask[*j] && !picks.contains(j)) else { break };
            picks.push(j);
        }
        g.answer(&sel, &picks).unwrap();
    }

    /// A game during setup in which player 0 has put down an Active Pokémon and at least one Benched Pokémon face down
    /// and player 1 decides next (choosing their own Pokémon), before anything is turned face up.
    pub(crate) fn setup_game() -> Game {
        let mut names: Vec<&str> = Vec::new();
        for (n, k) in [("Duraludon PRE 69", 4), (MON, 4), ("Fan Rotom SCR 118", 4), ("Team Rocket's Mewtwo ex DRI 81", 3)] {
            names.extend(std::iter::repeat(n).take(k));
        }
        names.resize(60, "Metal Energy MEE 8");
        let deck: Vec<DefId> = names.iter().map(|n| def_of(n)).collect();
        for seed in 0..500u32 {
            let mut g = Game::new(seed);
            g.start([&deck, &deck]).unwrap();
            g.settle().unwrap();
            while g.st.phase == crate::types::GamePhase::Setup {
                let sel = g.select_with(false).unwrap().expect("a decision");
                let down = |p: usize| g.st.players[p].in_play().count();
                if sel.player == 1 && down(1) == 0 && down(0) >= 2 {
                    return g;
                }
                answer_all(&mut g);
            }
        }
        panic!("no seed gives the setup state");
    }

    /// Player 0's face-down cards: the Active Pokémon, then the Benched ones.
    fn face_down(g: &Game, p: usize) -> Vec<CardId> {
        g.st.players[p].in_play().map(|s| g.st.slot_pokemon(p, s).unwrap()).collect()
    }

    /// Whether every face-down spot of player `p` holds a card that may be put down there at setup.
    fn spots_fit(g: &Game, p: usize) -> bool {
        let pl = &g.st.players[p];
        pl.in_play().all(|s| {
            let d = g.st.cdef(g.st.slot_pokemon(p, s).unwrap());
            if s == pl.active { crate::engine::setup::may_start_active(d) } else { crate::engine::setup::may_bench_at_setup(d) }
        })
    }

    /// A matching is found where a greedy pass isn't: place 0 fits items 0 and 1, place 1 only item 0 (greedy gives item 0
    /// to place 0 and has nothing for place 1); none exists when two places fit only one item.
    #[test]
    fn fillable_is_a_matching() {
        let fit = |p: usize, i: usize| if p == 0 { i <= 1 } else { i == 0 };
        assert!(fillable(2, 2, &fit));
        assert!(!fillable(2, 2, &|_, i| i == 0));
        assert!(fillable(0, 0, &|_, _| false));
    }

    /// The other player's face-down Pokémon are hidden from a viewer (`unseen_cards`, `swap_hidden`); their owner sees
    /// them, and no player-less swap takes them.
    #[test]
    fn face_down_pokemon_are_hidden_from_the_other_player() {
        let mut g = setup_game();
        let down = face_down(&g, 0);
        let hidden = g.unseen_cards(1, 0);
        assert!(down.iter().all(|c| hidden.contains(c)));
        assert_eq!(hidden.len(), g.st.players[0].deck().len() + g.st.players[0].hand().len() + 6 + down.len());
        assert!(g.unseen_cards(0, 0).iter().all(|c| !down.contains(c)));
        let top = g.st.players[0].deck().as_slice()[0];
        assert_eq!(g.swap_hidden(0, down[0], top), Err(GameError("CARD_NOT_HIDDEN")));
        assert_eq!(g.swap_unseen(down[0], top), Err(GameError("CARD_NOT_UNSEEN")));
        // A card that may not be put down there is refused; a Basic Pokémon takes the spot (the spot's Pokémon follows).
        let energy = g.st.players[0].deck().iter().find(|c| g.st.cdef(*c).is_energy()).unwrap();
        assert_eq!(g.swap_hidden(1, down[0], energy), Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD")));
        assert_eq!(g.swap_hidden(1, energy, down[1]), Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD")));
        let basic = g.st.players[0].deck().iter().find(|c| crate::engine::setup::may_bench_at_setup(g.st.cdef(*c))).unwrap();
        let at = g.st.players[0].deck().index_of(basic).unwrap();
        g.swap_hidden(1, down[0], basic).unwrap();
        assert_eq!(g.st.active_pokemon(0), Some(basic));
        assert_eq!(g.st.players[0].deck().as_slice()[at], down[0]);
        assert_eq!(g.locate_card(basic).unwrap().zone, Zone::InPlay { spot: crate::place::Spot::Active, role: crate::place::InPlayAs::Pokemon });
        crate::invariants::check(&g).unwrap();
    }

    /// Determinizing for the other player re-deals the face-down Pokémon with the hidden cards, each spot keeping a card
    /// that may be put down there; for their owner they stay.
    #[test]
    fn determinize_redeals_face_down_pokemon_into_fitting_spots() {
        let g0 = setup_game();
        let mut mine = g0.fork();
        determinize(&mut mine, 0, 3).unwrap();
        assert_eq!(face_down(&mine, 0), face_down(&g0, 0));
        let mut changed = false;
        for seed in 1..40 {
            let mut g = g0.fork();
            determinize(&mut g, 1, seed).unwrap();
            assert!(spots_fit(&g, 0));
            assert_eq!(g.st.players[0].in_play().count(), g0.st.players[0].in_play().count());
            changed |= face_down(&g, 0) != face_down(&g0, 0);
            crate::invariants::check(&g).unwrap();
        }
        assert!(changed, "the face-down Pokémon are re-dealt");
    }

    /// The list must give every face-down spot a card that may be put down there: met (the Pokémon land in the spots),
    /// met only by a matching (the one card that may only start as the Active Pokémon listed after the Basic Pokémon a
    /// greedy pass would give the Active Spot), and impossible (refused, nothing changed).
    #[test]
    fn set_unseen_fills_face_down_spots_with_fitting_cards() {
        assert!(crate::engine::setup::may_start_active(crate::carddb::def(def_of(CINDERACE))));
        assert!(!crate::engine::setup::may_bench_at_setup(crate::carddb::def(def_of(CINDERACE))));
        let g0 = setup_game();
        let n = g0.unseen_cards(1, 0).len();
        let benched = g0.st.players[0].bench.iter().filter(|b| !g0.st.slot(0, **b).cards().is_empty()).count();
        // Met: Duraludon for every spot.
        let mut list: Vec<DefId> = vec![def_of("Duraludon PRE 69"); benched + 1];
        list.resize(n, def_of("Metal Energy MEE 8"));
        let mut g = g0.fork();
        g.set_unseen(1, 0, &list, 5).unwrap();
        assert!(face_down(&g, 0).iter().all(|c| g.st.cdef(*c).full_name == "Duraludon PRE 69"));
        crate::invariants::check(&g).unwrap();
        // Only a matching: the Basic Pokémon first, then Cinderace; Cinderace must start Active, the Basics be Benched.
        let mut list: Vec<DefId> = vec![def_of(MON); benched];
        list.push(def_of(CINDERACE));
        list.resize(n, def_of("Metal Energy MEE 8"));
        for seed in 0..10 {
            let mut g = g0.fork();
            g.set_unseen(1, 0, &list, seed).unwrap();
            assert_eq!(g.st.cdef(g.st.active_pokemon(0).unwrap()).full_name, CINDERACE);
            assert!(face_down(&g, 0)[1..].iter().all(|c| g.st.cdef(*c).full_name == MON));
            assert!(spots_fit(&g, 0));
            crate::invariants::check(&g).unwrap();
        }
        // Impossible: no card for a Bench spot (Cinderace may not be Benched at setup).
        let mut list: Vec<DefId> = vec![def_of(CINDERACE); benched + 1];
        list.resize(n, def_of("Metal Energy MEE 8"));
        let mut g = g0.fork();
        let (hash, cards) = (g.state_hash(), g.st.cards.map(|c| c.def));
        assert_eq!(g.set_unseen(1, 0, &list, 1), Err(GameError("FACE_DOWN_SPOT_UNFILLABLE")));
        assert_eq!((g.state_hash(), g.st.cards.map(|c| c.def)), (hash.clone(), cards));
        // Assigned in place: a card that may not be put down in the spot is refused; a Basic Pokémon is taken, and the
        // spot's Pokémon is the card as it now is.
        let active = g.st.active_pokemon(0).unwrap();
        assert_eq!(g.set_unseen_assigned(1, 0, &[(active, def_of("Metal Energy MEE 8"))]), Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD")));
        let benched = face_down(&g, 0)[1];
        assert_eq!(g.set_unseen_assigned(1, 0, &[(benched, def_of(CINDERACE))]), Err(GameError("FACE_DOWN_SPOT_REFUSES_CARD")));
        assert_eq!((g.state_hash(), g.st.cards.map(|c| c.def)), (hash, cards));
        g.set_unseen_assigned(1, 0, &[(active, def_of(CINDERACE))]).unwrap();
        assert_eq!(g.st.active_pokemon(0), Some(active));
        assert_eq!(g.st.cdef(active).full_name, CINDERACE);
        crate::invariants::check(&g).unwrap();
    }
}
