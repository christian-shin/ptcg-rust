//! The host API: the one module a program that embeds the engine (an adapter, a training loop, a harness) is meant to
//! use. It re-exports the types a host reads and wraps the calls it makes in thin functions, so what a host depends on
//! is listed here and documented once. Nothing in this module adds behavior: every function is a call of the engine's
//! own, and every guarantee below is the guarantee of that call.
//!
//! A game is driven in a loop: [`start_game`], then [`next`] to learn what the game waits for, then [`answer`] (a prompt)
//! or [`act`] (a turn action), until [`Decision::Over`]. Between decisions a host reads [`view`], [`locate_card`], the
//! game record ([`take_record`]) and the chance counters ([`tape_misses`], [`tape_left`]).
//!
//! Threads. A [`Game`] is plain data (`Send + Clone`). The chance tape, the chance recording ([`take_chance`]) and the game
//! record are per-thread buffers: drive one recording game per thread, and read them on the thread that plays.
//!
//! Hidden information. [`view`] shows what a seat may see; [`Decision`] and [`PromptRec`] are the engine's own view of a
//! decision (a host that serves a seat must not show it to the other seat); [`swap_unseen`] and [`determinize`] change
//! only cards no seat can see.

pub use crate::carddb::{AttackDef, CardDef, DefId, PowerDef};
pub use crate::game::{Action, Game, GameError, Pending};
pub use crate::interface::{AreaType, Opt, OptionType, SelectContext, SelectData, SelectType};
pub use crate::list::CardId;
pub use crate::place::{CardPlace, InPlayAs, Spot, Zone};
pub use crate::prompts::{PromptKind, PromptPurpose, PromptRec, Provenance};
pub use crate::record::{Detail, RecordEntry};
pub use crate::rng::Draw;
pub use crate::spec::passive::RuleSource;
pub use crate::state::NO_PLAYER;
pub use crate::types::WINNER_NONE;
pub use crate::view::{CardView, EnergyUnit, EnergyUnitKind, EnergyView, PokemonView, SeatView, TableView};

use crate::game::R;
use std::collections::HashMap;
use std::sync::OnceLock;

/// How to start a game ([`start_game`]).
#[derive(Clone, Debug, Default)]
pub struct StartOptions {
    /// Chance outcomes to feed back instead of drawing them (a replay): the first outcome of each draw's kind and size is
    /// taken ([`set_tape`]). `None`: every chance outcome is drawn from the seeded stream.
    pub tape: Option<Vec<Draw>>,
    /// Record the chance outcomes this game draws, in order ([`take_chance`]).
    pub record_chance: bool,
    /// Record the game ([`take_record`]) from the first setup step on.
    pub record_events: bool,
}

/// What the game waits for ([`next`]).
#[derive(Clone, Debug)]
pub enum Decision {
    /// The game is over. `winner` is the winning seat, `WINNER_NONE` for none, or a draw (`Game::st.winner`).
    Over { winner: i8 },
    /// A turn decision of `player`: one of `actions` goes to [`act`] (the engine's own list of legal turn actions, in its
    /// order; never empty, `Action::Pass` ends the turn).
    Turn { player: u8, actions: Vec<Action> },
    /// A question of `player`: the prompt record (why it is asked and by which card) and its options; answered by option
    /// indices through [`answer`]. [`pick_mask`] gives the valid next picks.
    Prompt { player: u8, prompt: PromptRec, select: SelectData },
}

impl Decision {
    /// The seat that decides (`None` when the game is over).
    pub fn player(&self) -> Option<u8> {
        match self {
            Decision::Over { .. } => None,
            Decision::Turn { player, .. } | Decision::Prompt { player, .. } => Some(*player),
        }
    }
}

/// A game of `decks` (each a list of card definitions, in the order the cards are numbered: the card ids of seat `s` are
/// `60 * s + position` for 60-card decks) with the chance stream of `seed`, started and settled to its first decision.
///
/// Guarantees: the same decks, seed and tape give the same game. A deck that breaks the deck-building rules ends the game
/// at once (the first [`next`] is `Decision::Over` with `winner == WINNER_NONE`). Setting a tape or chance recording
/// affects the calling thread (see the module note); an earlier tape of the thread is replaced.
pub fn start_game(decks: [&[DefId]; 2], seed: u32, opts: &StartOptions) -> R<Game> {
    let mut g = Game::new(seed);
    set_tape(opts.tape.clone());
    if opts.record_chance {
        g.rng = g.rng.record();
        crate::rng::take_recorded();
    }
    if opts.record_events {
        g.record_on();
    }
    g.start(decks)?;
    g.settle()?;
    Ok(g)
}

/// What the game waits for now. Chance and information prompts are resolved first (their outcomes come from the tape,
/// then the seeded stream); the game changes only by that. A game that is stuck (no decision, not over) is an error
/// `STUCK`, which no legal game reaches.
pub fn next(g: &mut Game) -> R<Decision> {
    g.settle()?;
    match g.pending() {
        Pending::Finished => Ok(Decision::Over { winner: g.st.winner }),
        Pending::Stuck => Err(GameError("STUCK")),
        Pending::Turn(player) => {
            let mut actions = Vec::new();
            crate::options::legal_actions_into(g, &mut actions);
            Ok(Decision::Turn { player, actions })
        }
        Pending::Decision(i) => {
            let prompt = g.prompts.as_slice()[i];
            let select = g.select()?.ok_or(GameError("STUCK"))?;
            Ok(Decision::Prompt { player: prompt.player, prompt, select })
        }
        Pending::Chance(_) | Pending::Info(_) => unreachable!("settled"),
    }
}

/// The valid next picks of a prompt's `select` after `picks` (as [`Game::pick_mask`]): one flag per option, and whether
/// the picks so far already form a complete answer. Does not change the game.
pub fn pick_mask(g: &Game, select: &SelectData, picks: &[usize]) -> (Vec<bool>, bool) {
    g.pick_mask(select, picks)
}

/// Answer a prompt with option indices (as [`Game::answer`]) and settle chance. An answer the prompt does not accept is
/// an error and leaves the game unchanged.
pub fn answer(g: &mut Game, select: &SelectData, picks: &[usize]) -> R {
    g.answer(select, picks)?;
    g.settle()
}

/// Take a turn action (one of the [`Decision::Turn`] list) and settle chance. An action the rules do not allow is an
/// error and leaves the game unchanged.
pub fn act(g: &mut Game, action: Action) -> R {
    g.act(action)?;
    g.settle()
}

/// What `seat` may see of the table ([`TableView::new`]): its own hand and the public zones, the opponent's counts.
pub fn view(g: &Game, seat: usize) -> TableView {
    TableView::new(g, seat)
}

/// Where card `c` is ([`Game::locate_card`]); `None` for a number that is no card of this game.
pub fn locate_card(g: &Game, c: CardId) -> Option<CardPlace> {
    g.locate_card(c)
}

/// The card definition behind card `c` of the game (its printed data: name, set, attacks, powers).
pub fn def_of(g: &Game, c: CardId) -> &'static CardDef {
    crate::carddb::def(g.st.cards[c as usize].def)
}

/// Whether a printed power is an Ability or a card's own rule action ([`PowerDef::source`]).
pub fn power_source(p: &PowerDef) -> RuleSource {
    p.source()
}

/// Record the game from now on ([`Game::record_on`]; call before `start` to see the setup: [`StartOptions::record_events`]).
pub fn record_on(g: &mut Game) {
    g.record_on();
}

/// The game-record entries made since the last call (between decisions every event is done), oldest first; empty when
/// the record is off ([`Game::take_record`]).
pub fn take_record(g: &mut Game) -> Vec<RecordEntry> {
    g.take_record()
}

/// Replace this thread's chance tape (`None` clears it); the miss count starts at 0 ([`crate::rng::set_tape`]).
pub fn set_tape(tape: Option<Vec<Draw>>) {
    crate::rng::set_tape(tape);
}

/// Draws since the tape was set that it had no outcome of their kind and size for (0 for a faithful replay and without a
/// tape; [`crate::rng::tape_misses`]).
pub fn tape_misses() -> usize {
    crate::rng::tape_misses()
}

/// Outcomes on the tape no draw has taken yet (0 without a tape; [`crate::rng::tape_left`]).
pub fn tape_left() -> usize {
    crate::rng::tape_left()
}

/// The chance outcomes drawn on this thread since the last call by a game that records them
/// ([`StartOptions::record_chance`]); feed them back as a tape to replay.
pub fn take_chance() -> Vec<Draw> {
    crate::rng::take_recorded()
}

/// Trade the places of two cards that no seat can see (two deck cards, or two face-down Prize cards, of one owner);
/// anything else is an error `CARD_NOT_UNSEEN` and changes nothing ([`Game::swap_unseen`]).
pub fn swap_unseen(g: &mut Game, a: CardId, b: CardId) -> R {
    g.swap_unseen(a, b)
}

/// Rearrange the cards `viewer` cannot see as one consistent guess ([`crate::search::determinize`]), shuffled with `seed`;
/// what the viewer can see is untouched.
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    crate::search::determinize(g, viewer, seed)
}

/// Every card definition of the table, indexed by [`DefId`] ([`crate::carddb::cards`]).
pub fn cards() -> &'static [CardDef] {
    crate::carddb::cards()
}

/// The card with printing `number` of set `set` (international set code and printing number as printed: `"SCR"`, `"136"`),
/// `None` if the table has none. Unique: no two cards share a set and number.
pub fn card_by_set_number(set: &str, number: &str) -> Option<DefId> {
    static INDEX: OnceLock<HashMap<(&'static str, &'static str), DefId>> = OnceLock::new();
    INDEX
        .get_or_init(|| cards().iter().enumerate().map(|(i, c)| ((c.set, c.set_number), i as DefId)).collect())
        .get(&(set, number))
        .copied()
}

/// The card with this full name ("Grand Tree SCR 136", or "Name SET" when one printing of the set has the name).
pub fn card_by_name(name: &str) -> Option<DefId> {
    crate::carddb::def_by_full_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::CardList;

    fn deck() -> Vec<DefId> {
        (0..60)
            .map(|i| match i {
                0..=3 => "Duraludon PRE 69",
                4..=6 => "Team Rocket's Mewtwo ex DRI 81",
                7..=10 => "Sacred Charm PFL 93",
                _ => "Metal Energy MEE 8",
            })
            .map(|n| card_by_name(n).unwrap())
            .collect()
    }

    /// Plays to the end through `ptcg::host` alone: prompts answered from the masks, turns by the first action that is
    /// not a pass (a pass at every fourth turn), with the record read on the way.
    fn play(seed: u32, tape: Option<Vec<Draw>>) -> (Game, Vec<Draw>, usize, usize) {
        let d = deck();
        let opts = StartOptions { tape, record_chance: true, record_events: true };
        let mut g = start_game([&d, &d], seed, &opts).unwrap();
        let mut entries = 0;
        let mut turns = 0usize;
        for _ in 0..20_000 {
            match next(&mut g).unwrap() {
                Decision::Over { .. } => break,
                Decision::Turn { player, actions } => {
                    assert!(matches!(g.pending(), Pending::Turn(p) if p == player));
                    assert!(!actions.is_empty() && actions.contains(&Action::Pass));
                    turns += 1;
                    let a = actions.iter().copied().find(|a| !matches!(a, Action::Pass)).filter(|_| turns % 4 != 0).unwrap_or(Action::Pass);
                    act(&mut g, a).unwrap();
                }
                Decision::Prompt { player, prompt, select } => {
                    assert_eq!(prompt.player, player);
                    assert_eq!(select.player, player);
                    let mut picks = Vec::new();
                    loop {
                        let (mask, done) = pick_mask(&g, &select, &picks);
                        if picks.len() >= select.min_count && (done || picks.len() >= select.max_count) {
                            break;
                        }
                        picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
                    }
                    answer(&mut g, &select, &picks).unwrap();
                }
            }
            entries += take_record(&mut g).len();
        }
        let chance = take_chance();
        (g, chance, entries, turns)
    }

    /// A whole game runs through the facade; the record has entries; replaying its chance outcomes as a tape gives the
    /// same game with no miss and nothing left; a viewer's table and a card's place are readable.
    #[test]
    fn a_full_game_is_driven_only_through_the_host_api() {
        let (g, chance, entries, turns) = play(21, None);
        assert!(matches!(g.pending(), Pending::Finished), "the game did not finish");
        assert!(entries > 100 && turns > 4 && !chance.is_empty());
        let (g2, _, _, _) = play(21, Some(chance));
        assert_eq!(tape_misses(), 0);
        assert_eq!(tape_left(), 0);
        assert_eq!(g.state_hash(), g2.state_hash());
        set_tape(None);
        for seat in 0..2 {
            let v = view(&g2, seat);
            assert_eq!(v.viewer as usize, seat);
            if let Some(c) = v.me().discard.iter().next() {
                assert_eq!(locate_card(&g2, c.card).unwrap().owner as usize, seat);
                assert!(std::ptr::eq(def_of(&g2, c.card), crate::carddb::def(c.def)));
            }
        }
    }

    /// Cards are found by set and number, and the pair is unique over the table.
    #[test]
    fn a_card_is_found_by_set_and_number() {
        for (i, c) in cards().iter().enumerate() {
            assert_eq!(card_by_set_number(c.set, c.set_number), Some(i as DefId), "{}", c.full_name);
        }
        assert_eq!(card_by_set_number("SCR", "136"), card_by_name("Grand Tree SCR 136"));
        assert_eq!(card_by_set_number("SCR", "no"), None);
    }

    /// Two cards of a deck that nobody can see trade places; a card in a hand does not.
    #[test]
    fn hidden_cards_swap_and_seen_ones_do_not() {
        let d = deck();
        let mut g = start_game([&d, &d], 3, &StartOptions::default()).unwrap();
        let (a, b) = (g.st.players[0].deck.get(0).unwrap(), g.st.players[0].deck.get(1).unwrap());
        let (pa, pb) = (locate_card(&g, a).unwrap(), locate_card(&g, b).unwrap());
        swap_unseen(&mut g, a, b).unwrap();
        assert_eq!(locate_card(&g, a).unwrap().index, pb.index);
        assert_eq!(locate_card(&g, b).unwrap().index, pa.index);
        if let Some(h) = g.st.players[0].hand.get(0) {
            assert!(swap_unseen(&mut g, a, h).is_err());
        }
    }

    /// Abilities and card rules are told apart by `power_source`.
    #[test]
    fn a_power_names_its_source() {
        let fossil = cards().iter().find(|c| c.full_name == "Antique Root Fossil SCR 130").unwrap();
        assert_eq!(power_source(&fossil.powers[0]), RuleSource::CardRule);
    }
}
