//! Where a card is, in the rules' terms: its zone, the zone's owner and its position there ([`Game::locate_card`]).
//!
//! A host that names a card by its location (an option's card, a card in the record of the game) reads it here instead
//! of from a prompt's own list, which may be a scratch copy (a hand without the card being played, the cards set aside
//! for a search). The positions follow the orders the table view gives (`view.rs`), so a place points into the matching
//! `TableView` list.

use crate::game::Game;
use crate::list::{CardId, CardList};
use crate::spec::event::RulesZone;
use crate::state::{ListRef, SlotId};

/// A Pokémon's spot in play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Active,
    /// The Bench, by position (the order of `Player::bench`, `CardTarget::index`, `SeatView::bench`).
    Bench(u8),
}

/// What a card in a Pokémon's spot is there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InPlayAs {
    /// The Pokémon itself (the top card of the stack).
    Pokemon,
    /// A Pokémon card under the top card (its evolution line).
    PreEvolution,
    /// An attached Energy card (also a card attached as Energy).
    Energy,
    /// An attached Pokémon Tool.
    Tool,
}

/// A zone of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Deck,
    Hand,
    Discard,
    LostZone,
    Prizes,
    Stadium,
    /// A Trainer card being played: out of the hand, its effect resolving (discarded when it is done).
    PlayArea,
    /// Set aside while a player looks at it or searches for it (never a zone of the rules: the card is still in the
    /// zone `from` for every rule).
    Looking { from: RulesZone },
    /// In a Pokémon's spot.
    InPlay { spot: Spot, role: InPlayAs },
}

/// Where a card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardPlace {
    /// The player whose zone (or spot) it is; for a card set aside, the card's owner.
    pub owner: u8,
    pub zone: Zone,
    /// The card's position in the zone:
    /// - `Deck`: from the top (0 = the next card drawn);
    /// - `Hand`: hand order; `Discard`, `LostZone`, `PlayArea`: oldest first;
    /// - `Prizes`: among the Prize cards left (the order of `SeatView::prizes` and of the Prize prompts' options);
    /// - `Stadium`: 0;
    /// - `Looking`: among every card set aside, in the order `TableView::looking` lists them (the cards the viewer may
    ///   not see counted too);
    /// - `InPlay`: the Pokémon 0; a Pre-Evolution card lowest Stage first (`PokemonView::pre_evolution`); an Energy or a
    ///   Tool among the Pokémon's attached Energy or Tools in the order they arrived (`PokemonView::energy`, `tools`).
    pub index: u8,
}

impl Game {
    /// Where card `c` is; `None` for a card that is nowhere on the table (not a card of this game).
    pub fn locate_card(&self, c: CardId) -> Option<CardPlace> {
        if c >= self.st.n_cards {
            return None;
        }
        let st = &self.st;
        let at = |owner: u8, zone: Zone, index: usize| Some(CardPlace { owner, zone, index: index as u8 });
        let pos = |l: &[CardId]| l.iter().position(|x| *x == c).unwrap_or(0);
        match st.locate(c) {
            Some(ListRef::Deck(p)) => return at(p, Zone::Deck, pos(st.players[p as usize].deck.as_slice())),
            Some(ListRef::Hand(p)) => return at(p, Zone::Hand, pos(st.players[p as usize].hand.as_slice())),
            Some(ListRef::Discard(p)) => return at(p, Zone::Discard, pos(st.players[p as usize].discard.as_slice())),
            Some(ListRef::LostZone(p)) => return at(p, Zone::LostZone, pos(st.players[p as usize].lostzone.as_slice())),
            Some(ListRef::Stadium(p)) => return at(p, Zone::Stadium, 0),
            Some(ListRef::Supporter(p)) => return at(p, Zone::PlayArea, pos(st.players[p as usize].supporter.as_slice())),
            Some(ListRef::Prize(p, i)) => {
                let pl = &st.players[p as usize];
                let left = (0..i as usize).filter(|k| !pl.prizes[*k].is_empty()).count();
                return at(p, Zone::Prizes, left);
            }
            Some(ListRef::Slot(p, s)) => return self.spot_place(p, s, c),
            Some(ListRef::SlotEnergies(..) | ListRef::Temp(_)) | None => {}
        }
        // An attached Tool is in its Pokémon's Tools only; a card attached as Energy may be in its Energy list only.
        if let Some((p, s)) = st.tool_spot(c) {
            return self.spot_place(p as u8, s, c);
        }
        for p in 0..2u8 {
            let pl = &st.players[p as usize];
            for s in std::iter::once(pl.active).chain(pl.bench.iter().copied()) {
                if pl.slots[s as usize].energies().contains(c) {
                    return self.spot_place(p, s, c);
                }
            }
        }
        let inst = &st.cards[c as usize];
        let i = inst.staged_in?;
        if !(self.temp_used[i as usize] && self.temps[i as usize].contains(c)) {
            return None;
        }
        at(inst.owner, Zone::Looking { from: inst.staged_from.unwrap_or(RulesZone::Deck) }, self.looking_index(c))
    }

    /// The place of card `c` in the spot `s` of player `p`.
    fn spot_place(&self, p: u8, s: SlotId, c: CardId) -> Option<CardPlace> {
        let st = &self.st;
        let pl = &st.players[p as usize];
        let spot = if s == pl.active { Spot::Active } else { Spot::Bench(pl.bench.iter().position(|b| *b == s)? as u8) };
        let slot = st.slot(p as usize, s);
        let arrival_index = |l: &[CardId]| l.iter().filter(|x| st.cards[**x as usize].arrived < st.cards[c as usize].arrived).count();
        let (role, index) = if slot.tools().contains(c) {
            (InPlayAs::Tool, arrival_index(slot.tools().as_slice()))
        } else if slot.energies().contains(c) {
            (InPlayAs::Energy, arrival_index(slot.energies().as_slice()))
        } else if st.slot_pokemon(p as usize, s) == Some(c) {
            (InPlayAs::Pokemon, 0)
        } else {
            let under: crate::list::SVec<CardId, 8> = st.slot_pokemons(p as usize, s);
            (InPlayAs::PreEvolution, under.iter().position(|x| *x == c)?)
        };
        Some(CardPlace { owner: p, zone: Zone::InPlay { spot, role }, index: index as u8 })
    }

    /// The position of the set-aside card `c` among every card set aside, in the order `TableView::looking` lists them:
    /// the staging lists in use in order, each card in the list it was last moved into, once.
    fn looking_index(&self, c: CardId) -> usize {
        let mut seen: crate::list::SVec<CardId, 120> = crate::list::SVec::new();
        for i in 0..self.temp_hw as usize {
            if !self.temp_used[i] {
                continue;
            }
            for x in self.temps[i].iter() {
                if self.st.cards[x as usize].staged_in != Some(i as u8) || seen.contains(&x) {
                    continue;
                }
                if x == c {
                    return seen.len();
                }
                seen.push(x);
            }
        }
        seen.len()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::game::Pending;
    use crate::prompts::PromptKind;
    use serde_json::json;

    fn game(sc: serde_json::Value) -> Game {
        game_of(&[("Duraludon PRE 69", 4), ("Archaludon ex SSP 130", 2), ("Hero's Cape TEF 152", 1), ("Buddy-Buddy Poffin TEF 144", 4)], sc)
    }

    /// A game of `cards` (name, copies) filled up with Metal Energy, each player, at the first turn decision (the setup prompts answered from
    /// the pick masks), with the scenario `sc` applied.
    pub(crate) fn game_of(cards: &[(&str, usize)], sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = cards.iter().flat_map(|(n, k)| std::iter::repeat_n(*n, *k)).collect();
        names.resize(60, "Metal Energy MEE 8");
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(5);
        g.start([&deck, &deck]).unwrap();
        while let Some(sel) = g.select_with(false).unwrap() {
            if sel.select_type == crate::interface::SelectType::Main {
                break;
            }
            let mut picks = Vec::new();
            while picks.len() < sel.min_count {
                let (mask, _) = g.pick_mask(&sel, &picks);
                picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
            }
            g.answer(&sel, &picks).unwrap();
        }
        assert!(matches!(g.pending(), Pending::Turn(_)));
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// Every card of the table is located where its list holds it, at the position the view's order gives.
    #[test]
    fn every_card_is_located_where_it_is() {
        let g = game(json!({"me": {"reset": true, "active": ["Duraludon PRE 69", "Archaludon ex SSP 130"], "active_tool": "Hero's Cape TEF 152", "active_energy": ["Metal Energy MEE 8", "Metal Energy MEE 8"], "bench": [{"card": "Duraludon PRE 69"}, {"card": "Duraludon PRE 69"}], "discard": ["Metal Energy MEE 8", "Buddy-Buddy Poffin TEF 144"], "hand": ["Duraludon PRE 69", "Metal Energy MEE 8"]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let p = g.st.active_player as usize;
        let st = &g.st;
        let pl = &st.players[p];
        for (k, c) in pl.deck.iter().enumerate() {
            assert_eq!(g.locate_card(c), Some(CardPlace { owner: p as u8, zone: Zone::Deck, index: k as u8 }));
        }
        for (k, c) in pl.hand.iter().enumerate() {
            assert_eq!(g.locate_card(c), Some(CardPlace { owner: p as u8, zone: Zone::Hand, index: k as u8 }));
        }
        for (k, c) in pl.discard.iter().enumerate() {
            assert_eq!(g.locate_card(c), Some(CardPlace { owner: p as u8, zone: Zone::Discard, index: k as u8 }));
        }
        let mut left = 0;
        for i in 0..pl.prize_count as usize {
            for c in pl.prizes[i].iter() {
                assert_eq!(g.locate_card(c), Some(CardPlace { owner: p as u8, zone: Zone::Prizes, index: left }));
            }
            left += !pl.prizes[i].is_empty() as u8;
        }
        let v = crate::view::TableView::new(&g, p);
        let active = v.players[p].active.unwrap();
        assert_eq!(g.locate_card(active.top.card).unwrap().zone, Zone::InPlay { spot: Spot::Active, role: InPlayAs::Pokemon });
        for (k, e) in active.energy.iter().enumerate() {
            assert_eq!(g.locate_card(e.card.card), Some(CardPlace { owner: p as u8, zone: Zone::InPlay { spot: Spot::Active, role: InPlayAs::Energy }, index: k as u8 }));
        }
        assert_eq!(active.tools.len(), 1);
        assert_eq!(g.locate_card(active.tools[0].card), Some(CardPlace { owner: p as u8, zone: Zone::InPlay { spot: Spot::Active, role: InPlayAs::Tool }, index: 0 }));
        assert_eq!(active.pre_evolution.len(), 1);
        assert_eq!(g.locate_card(active.pre_evolution[0].card), Some(CardPlace { owner: p as u8, zone: Zone::InPlay { spot: Spot::Active, role: InPlayAs::PreEvolution }, index: 0 }));
        for (i, b) in v.players[p].bench.iter().enumerate() {
            if let Some(b) = b {
                assert_eq!(g.locate_card(b.top.card), Some(CardPlace { owner: p as u8, zone: Zone::InPlay { spot: Spot::Bench(i as u8), role: InPlayAs::Pokemon }, index: 0 }));
            }
        }
        let opp = v.players[1 - p].active.unwrap().top.card;
        assert_eq!(g.locate_card(opp).unwrap().owner as usize, 1 - p);
        assert_eq!(g.locate_card(NO_SUCH_CARD), None);
        // Every card of the game is somewhere.
        for c in 0..g.st.n_cards {
            assert!(g.locate_card(c).is_some(), "card {c} located nowhere");
        }
    }

    const NO_SUCH_CARD: CardId = crate::list::NO_CARD;

    /// The options of a deck search are located in the deck, from the top, whatever list the prompt shows; cards set
    /// aside for a look are `Looking` with the zone they came from.
    #[test]
    fn a_search_s_options_are_located_in_the_deck() {
        const MON: &str = "Hop's Phantump ASC 95";
        let mut g = game_of(&[(MON, 4), ("Buddy-Buddy Poffin TEF 144", 4)], json!({"me": {"reset": true, "active": MON, "hand": ["Buddy-Buddy Poffin TEF 144"]}, "opp": {"reset": true, "active": MON}}));
        let me = g.st.active_player as usize;
        let hand = g.st.players[me].hand.as_slice().to_vec();
        let k = hand.iter().position(|c| g.st.cdef(*c).name == "Buddy-Buddy Poffin").unwrap();
        let play = crate::options::legal_actions(&g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
        g.act(play).unwrap();
        let sel = g.select().unwrap().unwrap();
        let Pending::Decision(i) = g.pending() else { panic!("the search's prompt") };
        assert!(matches!(g.prompts.as_slice()[i].kind, PromptKind::ChooseCards { .. }));
        assert!(!sel.options.is_empty());
        for o in &sel.options {
            let c = o.serial.unwrap();
            let place = g.locate_card(c).unwrap();
            assert_eq!(place.zone, Zone::Deck);
            assert_eq!(place.owner as usize, me);
            assert_eq!(g.st.players[me].deck.as_slice()[place.index as usize], c);
        }
        // The Item itself is in the play area while its effect resolves.
        let ball = hand[k];
        assert_eq!(g.locate_card(ball).unwrap().zone, Zone::PlayArea);
        // A card set aside (staged) for a look is Looking, from the deck.
        let top = g.st.players[me].deck.as_slice()[0];
        let t = g.alloc_temp(&[]);
        crate::engine::cards_zone::stage(&mut g, ListRef::Deck(me as u8), &[top], t, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, me as u8));
        assert_eq!(g.locate_card(top), Some(CardPlace { owner: me as u8, zone: Zone::Looking { from: RulesZone::Deck }, index: 0 }));
    }
}
