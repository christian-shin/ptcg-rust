//! `TableView`: what one player sees of the table, as plain data (events batch 9 C13b).
//!
//! A `Copy` struct built from the game for one viewer, with every fact read through the derived layer (HP, the Energy
//! provided) or from the state's own records. It is the one source of the observation features (`obs.rs`), and the shape
//! an adapter to another observation format reads. It holds no reference to the game: build it, then read it anywhere.
//!
//! Seats are player indexes (`players[i]` is player `i`, not "you / opponent"); `viewer` says whose eyes. Hidden
//! information is absent, not masked: the opponent's hand is `None`, the Prize cards face down are `None`, the
//! opponent's Pokémon put down face down at setup are `None` (with the spot marked face down), no deck content exists at
//! all. Lists keep the order the rules give: the discard pile and the Lost Zone oldest first, the hand
//! in hand order, a Pokémon's attached cards in the order they arrived (`CardInst::arrived`).

use crate::carddb::DefId;
use crate::game::Game;
use crate::list::*;
use crate::state::*;
use crate::types::*;

/// A card as the viewer names it: its id in this game (the serial) and its definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardView {
    /// The card's id in this game (`CardId`, unique per game; cabt's "serial").
    pub card: CardId,
    /// The card's definition (`carddb`), what the card is.
    pub def: DefId,
}

/// An Energy card attached to a Pokémon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnergyView {
    pub card: CardView,
    /// The first Energy type the card's text provides (`COLORLESS` when it provides none): the type a summary of the
    /// board counts the card under.
    pub first_type: CardType,
}

/// One unit of Energy a Pokémon provides, and the card that provides it (an attached Energy card; the card of an effect
/// that makes the Pokémon provide Energy).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyUnit {
    /// The card providing the unit (`CardView::card`: its id in this game).
    pub card: CardId,
    pub kind: EnergyUnitKind,
}

/// What one unit of Energy is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnergyUnitKind {
    /// One unit of this type (`ct::ANY` for an Energy that provides every type, Prism Energy).
    One(CardType),
    /// One unit of whichever of these types its holder chooses (Team Rocket's Energy).
    AnyOf(SVec<CardType, 4>),
}

/// One Pokémon in play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PokemonView {
    /// The top card of the spot's stack: the Pokémon itself.
    pub top: CardView,
    /// Its HP with every modifier applied (the derived layer), before damage.
    pub max_hp: i32,
    /// The damage on it, in HP (damage counters times 10).
    pub damage: i32,
    /// The HP left: `max_hp - damage`.
    pub hp: i32,
    /// The turn it came into play (put into play, evolved or devolved; `Slot::entered_turn`).
    pub entered_turn: i32,
    /// It came into play this turn.
    pub entered_this_turn: bool,
    /// The Energy it provides, one entry per unit (`energy::unit_count`), in the derived layer's Energy map order: a card
    /// that provides a choice of types (Team Rocket's Energy) is one unit, not one per type.
    pub provided: SVec<EnergyUnit, 64>,
    /// The Energy cards attached, in arrival order.
    pub energy: SVec<EnergyView, 60>,
    /// The Tool cards attached, in arrival order.
    pub tools: SVec<CardView, 4>,
    /// The Pokémon cards under the top card (its evolution line, lowest Stage first).
    pub pre_evolution: SVec<CardView, 8>,
    /// The Special Conditions on it, bit `c` set for condition `c` (`Slot::special_conditions`).
    pub conditions: u32,
}

/// One seat of the table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeatView {
    /// Cards in the deck (their identity is never shown).
    pub deck_count: u8,
    /// Cards in the hand.
    pub hand_count: u8,
    /// The hand in hand order: the viewer's own only (`None` for the opponent's).
    pub hand: Option<SVec<CardView, 120>>,
    /// The discard pile, oldest first.
    pub discard: SVec<CardView, 120>,
    /// The Lost Zone, oldest first.
    pub lost_zone: SVec<CardView, 120>,
    /// Prize cards left.
    pub prize_count: u8,
    /// The Prize cards left, `Some` only for a card turned face up (the list has `prize_count` entries; a Prize card with
    /// no card yet, none here, counts as face down).
    pub prizes: SVec<Option<CardView>, 6>,
    /// The Active Pokémon, `None` before one is chosen, and to the other player while it lies face down
    /// (`active_face_down`).
    pub active: Option<PokemonView>,
    /// The Active Spot holds a card face down (put down at setup, not turned face up yet: `State::in_play_face_down`).
    /// Its owner sees the card in `active`; the other player sees only that the spot holds one.
    pub active_face_down: bool,
    /// The Bench: one entry per Bench spot, in order, `None` for an empty spot and, to the other player, for a spot
    /// holding a card face down (`bench_face_down`) (`bench_max` entries are meaningful).
    pub bench: [Option<PokemonView>; MAX_BENCH],
    /// Each Bench spot holding a card face down (as `active_face_down`), in the order of `bench`.
    pub bench_face_down: [bool; MAX_BENCH],
    /// The number of Bench spots (5 unless a card changes it), filled or not.
    pub bench_max: u8,
    /// A Supporter was played from the hand this turn (this seat's record of the turn).
    pub supporter_played: bool,
    /// A Stadium was played from the hand this turn.
    pub stadium_played: bool,
    /// An Energy was attached from the hand this turn (the once-a-turn attachment).
    pub energy_attached: bool,
    /// This seat retreated this turn.
    pub retreated: bool,
    /// The VSTAR Power / GX attack of the game was used.
    pub used_vstar: bool,
    pub used_gx: bool,
}

/// The table as `viewer` sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableView {
    /// The seat looking.
    pub viewer: u8,
    /// The turn number (0 during setup).
    pub turn: i32,
    /// The phase of the game (`GamePhase` as a number).
    pub phase: u8,
    /// The seat whose turn it is.
    pub active_player: u8,
    /// The seat that took the first turn, `NO_PLAYER` while undecided.
    pub first_player: u8,
    /// The Stadium in play and its owner.
    pub stadium: Option<(CardView, u8)>,
    /// The cards the viewer is looking at: the cards set aside for a look or a search that the viewer may see (their own
    /// cards, cards from a public zone, and any the viewer is the one deciding about).
    /// A card the viewer may not see is `None` (its place in the set-aside list is shown, its identity is not).
    pub looking: SVec<Option<CardView>, 120>,
    pub players: [SeatView; 2],
}

fn card(g: &Game, c: CardId) -> CardView {
    CardView { card: c, def: g.st.cards[c as usize].def }
}

fn cards_of<const N: usize, const Z: bool>(g: &Game, l: &List<N, Z>) -> SVec<CardView, 120> {
    let mut v = SVec::new();
    for c in l.iter() {
        v.push(card(g, c));
    }
    v
}

fn pokemon(g: &Game, facts: &mut crate::legal::Ctx, p: usize, s: SlotId) -> Option<PokemonView> {
    let st = &g.st;
    let slot = st.slot(p, s);
    let top = st.slot_pokemon(p, s)?;
    let d = st.cdef(top);
    // The HP with its modifiers (the derived layer over the game, no memo).
    let max_hp = crate::derived::compute_hp(facts, p, s).unwrap_or(d.hp);
    let mut provided = SVec::new();
    for e in crate::derived::compute_provided_energy(facts, p, crate::effects::SlotRef::new(p, s)).unwrap_or_default().iter() {
        if crate::energy::is_choosable(e.provides.as_slice()) {
            provided.push(EnergyUnit { card: e.card, kind: EnergyUnitKind::AnyOf(e.provides) });
        } else {
            for t in e.provides.iter() {
                provided.push(EnergyUnit { card: e.card, kind: EnergyUnitKind::One(*t) });
            }
        }
    }
    let mut attached: SVec<CardId, 60> = SVec::new();
    for c in slot.energies().iter() {
        attached.push(c);
    }
    attached.as_mut_slice().sort_by_key(|c| st.cards[*c as usize].arrived);
    let mut energy = SVec::new();
    for &c in attached.iter() {
        energy.push(EnergyView { card: card(g, c), first_type: st.cdef(c).provides.first().copied().unwrap_or(ct::COLORLESS) });
    }
    let mut tool_cards: SVec<CardId, 4> = SVec::new();
    for c in slot.tools().iter() {
        tool_cards.push(c);
    }
    tool_cards.as_mut_slice().sort_by_key(|c| st.cards[*c as usize].arrived);
    let mut tools = SVec::new();
    for &c in tool_cards.iter() {
        tools.push(card(g, c));
    }
    let mut pre_evolution = SVec::new();
    for c in st.slot_pokemons(p, s).iter() {
        if *c != top {
            pre_evolution.push(card(g, *c));
        }
    }
    let mut conditions = 0u32;
    for c in slot.special_conditions.iter() {
        conditions |= 1 << c;
    }
    Some(PokemonView {
        top: card(g, top),
        max_hp,
        damage: slot.damage,
        hp: max_hp - slot.damage,
        entered_turn: slot.entered_turn,
        entered_this_turn: slot.entered_turn == st.turn,
        provided,
        energy,
        tools,
        pre_evolution,
        conditions,
    })
}

fn seat(g: &Game, facts: &mut crate::legal::Ctx, viewer: usize, p: usize) -> SeatView {
    let st = &g.st;
    let pl = &st.players[p];
    let trainer = |kind: TrainerType| pl.played_this_turn.iter().any(|c| st.cdef(*c).trainer_type == kind as u8);
    let mut prizes = SVec::new();
    for i in 0..pl.prize_count as usize {
        if pl.prizes[i].is_empty() {
            continue;
        }
        prizes.push(if pl.prize_face_up[i] { pl.prizes[i].get(0).map(|c| card(g, c)) } else { None });
    }
    // A card lying face down is seen by its owner only (`State::in_play_face_down`).
    let down = |s: SlotId| st.in_play_face_down() && !st.slot(p, s).cards().is_empty();
    let mut spot = |s: SlotId| if down(s) && p != viewer { None } else { pokemon(g, facts, p, s) };
    let mut bench = [None; MAX_BENCH];
    let mut bench_face_down = [false; MAX_BENCH];
    for (i, &s) in pl.bench.iter().enumerate().take(MAX_BENCH) {
        bench[i] = spot(s);
        bench_face_down[i] = down(s);
    }
    let active = spot(pl.active);
    SeatView {
        deck_count: pl.deck.len() as u8,
        hand_count: pl.hand.len() as u8,
        hand: if p == viewer { Some(cards_of(g, &pl.hand)) } else { None },
        discard: cards_of(g, &pl.discard),
        lost_zone: cards_of(g, &pl.lostzone),
        prize_count: pl.prize_left() as u8,
        prizes,
        active,
        active_face_down: down(pl.active),
        bench,
        bench_face_down,
        bench_max: pl.bench.len() as u8,
        supporter_played: trainer(TrainerType::Supporter),
        stadium_played: trainer(TrainerType::Stadium),
        energy_attached: pl.energy_played_turn == st.turn,
        retreated: pl.retreated_turn == st.turn,
        used_vstar: pl.used_vstar,
        used_gx: pl.used_gx,
    }
}

impl TableView {
    /// The table as `viewer` sees it.
    pub fn new(g: &Game, viewer: usize) -> TableView {
        let mut facts = crate::legal::Ctx::new(g);
        let st = &g.st;
        let stadium = (0..2).find_map(|q| st.players[q].stadium.get(0).map(|c| (card(g, c), q as u8)));
        TableView {
            viewer: viewer as u8,
            turn: st.turn,
            phase: st.phase as u8,
            active_player: st.active_player,
            first_player: st.first_player,
            stadium,
            looking: looking(g, viewer),
            players: [seat(g, &mut facts, viewer, 0), seat(g, &mut facts, viewer, 1)],
        }
    }

    /// The seat of the viewer.
    pub fn me(&self) -> &SeatView {
        &self.players[self.viewer as usize]
    }

    /// The other seat.
    pub fn opponent(&self) -> &SeatView {
        &self.players[1 - self.viewer as usize]
    }
}

/// The cards set aside for a look or a search (the staging lists in use) that `viewer` may see: the player looking sees
/// them, everyone sees a card taken from a public zone, and a card's owner sees it only if they are the one looking
/// (`record::look_seen_by`, the game record's rule). A card the viewer may not see keeps its place as `None`.
fn looking(g: &Game, viewer: usize) -> SVec<Option<CardView>, 120> {
    use crate::spec::event::RulesZone;
    let mut out: SVec<Option<CardView>, 120> = SVec::new();
    let mut seen: SVec<CardId, 120> = SVec::new();
    for i in 0..g.temp_hw as usize {
        if !g.temp_used[i] {
            continue;
        }
        for c in g.temps[i].iter() {
            let inst = &g.st.cards[c as usize];
            if inst.staged_in != Some(i as u8) || seen.iter().any(|&x| x == c) {
                continue;
            }
            seen.push(c);
            let by = inst.staged_by.map_or(inst.owner, |b| b as u8);
            let may = crate::record::look_seen_by(inst.staged_from.unwrap_or(RulesZone::Deck), by)[viewer];
            out.push(if may { Some(card(g, c)) } else { None });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hidden information is absent, the Pokémon's attached cards come in arrival order, and the view is a plain `Copy`.
    #[test]
    fn the_view_hides_and_orders() {
        fn is_copy<T: Copy>() {}
        is_copy::<TableView>();
        let names: Vec<&str> = (0..60).map(|i| if i < 4 { "Duraludon PRE 69" } else { "Metal Energy MEE 8" }).collect();
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(3);
        g.start([&deck, &deck]).unwrap();
        g.settle().unwrap();
        let sc = serde_json::json!({"me": {"reset": true, "active": "Duraludon PRE 69", "active_energy": ["Metal Energy MEE 8"], "bench": [{"card": "Duraludon PRE 69"}]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        let (a, b) = (g.st.players[0].active, g.st.players[0].bench.as_slice()[0]);
        for s in [b, a, b] {
            let c = g.st.players[0].deck.iter().find(|c| g.st.cdef(*c).is_energy()).unwrap();
            g.move_card_to(ListRef::Deck(0), c, ListRef::Slot(0, s));
        }
        let v = TableView::new(&g, 0);
        assert!(v.players[0].hand.is_some() && v.players[1].hand.is_none());
        assert!(TableView::new(&g, 1).players[0].hand.is_none());
        let active = v.players[0].active.unwrap();
        let stamps: Vec<u16> = active.energy.iter().map(|e| g.st.cards[e.card.card as usize].arrived).collect();
        assert_eq!(stamps.len(), 2);
        assert!(stamps.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(v.players[0].prizes.iter().filter(|p| p.is_some()).count(), 0);
        assert_eq!(v.players[0].prize_count, 6);
        assert_eq!(v.first_player, g.st.first_player);
    }

    /// During setup a player's Pokémon lie face down: their owner sees them, the other player sees only which spots hold
    /// a card (SSP rulebook p. 8 steps 5 and 6). After both are turned face up (step 8) both players see them.
    #[test]
    fn face_down_pokemon_are_seen_by_their_owner_only() {
        let mut g = crate::search::tests::setup_game();
        let pl = &g.st.players[0];
        let (active, bench0) = (g.st.active_pokemon(0).unwrap(), g.st.slot_pokemon(0, pl.bench.as_slice()[0]).unwrap());
        let (mine, theirs) = (TableView::new(&g, 0), TableView::new(&g, 1));
        for v in [&mine, &theirs] {
            let s = &v.players[0];
            assert!(s.active_face_down && s.bench_face_down[0]);
            assert!(!v.players[1].active_face_down && v.players[1].active.is_none(), "player 1 has put down nothing yet");
        }
        assert_eq!(mine.players[0].active.map(|p| p.top.card), Some(active));
        assert_eq!(mine.players[0].bench[0].map(|p| p.top.card), Some(bench0));
        assert!(theirs.players[0].active.is_none() && theirs.players[0].bench.iter().all(|b| b.is_none()));
        let down = mine.players[0].bench_face_down.iter().filter(|d| **d).count();
        assert_eq!(down, theirs.players[0].bench_face_down.iter().filter(|d| **d).count());
        assert_eq!(down, mine.players[0].bench.iter().filter(|b| b.is_some()).count());
        // The observation of the other player has no feature of a face-down Pokémon.
        let opp = crate::obs::GLOBAL_FEATURES + crate::obs::PLAYER_FEATURES + 16;
        assert!(crate::obs::observe(&g, 1)[opp..opp + crate::obs::SLOTS * crate::obs::SLOT_FEATURES].iter().all(|x| *x == 0.0));
        // Setup ends: both players' Pokémon are face up to both.
        while g.st.phase == crate::types::GamePhase::Setup {
            crate::search::tests::answer_all(&mut g);
        }
        let (mine, theirs) = (TableView::new(&g, 0), TableView::new(&g, 1));
        for p in 0..2 {
            assert_eq!(mine.players[p].active, theirs.players[p].active);
            assert_eq!(mine.players[p].bench, theirs.players[p].bench);
            assert!(mine.players[p].active.is_some() && !mine.players[p].active_face_down && !theirs.players[p].active_face_down);
            assert!(theirs.players[p].bench_face_down.iter().all(|d| !*d));
        }
        assert_eq!(theirs.players[0].active.map(|p| p.top.card), Some(active));
    }

    fn board(seed: u32, energy: &[&str]) -> Game {
        let names: Vec<&str> = (0..60).map(|i| if i < 4 { "Duraludon PRE 69" } else if i < 6 { "Team Rocket's Energy DRI 182" } else if i < 8 { "Prism Energy ASC 216" } else { "Metal Energy MEE 8" }).collect();
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(seed);
        g.start([&deck, &deck]).unwrap();
        g.settle().unwrap();
        let sc = serde_json::json!({"me": {"reset": true, "active": "Duraludon PRE 69", "active_energy": energy}, "opp": {"reset": true, "active": "Duraludon PRE 69"}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// `provided` has one entry per unit: a card that provides a choice of types is one `AnyOf` unit, an Energy that
    /// provides every type one `One(ANY)`, and its count is the sum of `energy::unit_count` over the attached cards.
    #[test]
    fn provided_energy_is_counted_per_unit() {
        let g = board(5, &["Metal Energy MEE 8", "Team Rocket's Energy DRI 182", "Prism Energy ASC 216"]);
        let pk = TableView::new(&g, 0).players[0].active.unwrap();
        let units: Vec<(EnergyUnitKind, CardId)> = pk.provided.iter().map(|u| (u.kind, u.card)).collect();
        // Each unit names its card: the attached Energy card of that name.
        for u in pk.provided.iter() {
            assert!(pk.energy.iter().any(|e| e.card.card == u.card), "unit of a card not attached");
        }
        // Metal Energy: one unit. Team Rocket's Energy: "2 in any combination of [P] and [D]", two units each a
        // choice of Psychic or Darkness (not four flattened types). Prism Energy: one unit of any type.
        let any_of = EnergyUnitKind::AnyOf({
            let mut v = SVec::new();
            v.push(ct::PSYCHIC);
            v.push(ct::DARK);
            v
        });
        let kinds: Vec<EnergyUnitKind> = units.iter().map(|u| u.0).collect();
        assert_eq!(kinds, vec![any_of, any_of, EnergyUnitKind::One(ct::ANY), EnergyUnitKind::One(ct::METAL)]);
        // Both Team Rocket's units come from one card; the others each from their own card, in the card's name.
        assert_eq!(units[0].1, units[1].1);
        let def_of = |c: CardId| g.st.cdef(c).name;
        assert_eq!(def_of(units[0].1), "Team Rocket's Energy");
        assert_eq!(def_of(units[2].1), "Prism Energy");
        assert_eq!(def_of(units[3].1), "Metal Energy");
    }

    /// A staged card the viewer may not see keeps its place in `looking` as `None`: looking at the top of the opponent's
    /// deck shows it to the looker only, never to its owner.
    #[test]
    fn looking_keeps_hidden_entries() {
        let seen = |v: &TableView| v.looking.iter().map(|c| c.map(|c| c.card)).collect::<Vec<_>>();
        let mut g = board(6, &[]);
        let theirs = g.st.players[1].deck.iter().next().unwrap();
        let mine = g.st.players[0].deck.iter().next().unwrap();
        let list = g.alloc_temp(&[]);
        // Player 0 looks at the top of the opponent's deck and of their own.
        let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, 0);
        crate::engine::cards_zone::stage(&mut g, ListRef::Deck(1), &[theirs], list, cause);
        crate::engine::cards_zone::stage(&mut g, ListRef::Deck(0), &[mine], list, cause);
        assert_eq!(seen(&TableView::new(&g, 0)), vec![Some(theirs), Some(mine)]);
        assert_eq!(seen(&TableView::new(&g, 1)), vec![None, None]);
        // The same look by player 1: the roles swap.
        let mut g = board(6, &[]);
        let theirs = g.st.players[1].deck.iter().next().unwrap();
        let list = g.alloc_temp(&[]);
        let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, 1);
        crate::engine::cards_zone::stage(&mut g, ListRef::Deck(1), &[theirs], list, cause);
        assert_eq!(seen(&TableView::new(&g, 1)), vec![Some(theirs)]);
        assert_eq!(seen(&TableView::new(&g, 0)), vec![None]);
    }
}
