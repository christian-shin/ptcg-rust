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
//! Lookahead. [`attack_what_if`] tells what an attack would do now (damage, Knock Outs, Prize cards) on copies of the
//! game, without playing the turn out; [`attack_what_if_with_energy`] with extra Energy attached to the attacker first.
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
pub use crate::what_if::{AttackOutcome, AttackWhatIf};

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

/// Trade the places of two cards of one owner that `viewer` cannot see (the other seat's hand, either deck, face-down
/// Prize cards no effect made public: the rule [`determinize`] re-deals by); anything else is an error
/// `CARD_NOT_HIDDEN` and changes nothing ([`Game::swap_hidden`]).
pub fn swap_hidden(g: &mut Game, viewer: usize, a: CardId, b: CardId) -> R {
    g.swap_hidden(viewer, a, b)
}

/// Rearrange the cards `viewer` cannot see as one consistent guess ([`crate::search::determinize`]), shuffled with `seed`;
/// what the viewer can see is untouched.
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    crate::search::determinize(g, viewer, seed)
}

/// Give the cards of `owner` that `viewer` cannot see (the places [`swap_hidden`] trades) the identities `cards`, as many
/// as there are such cards, dealt among those places at random with `seed`; what the viewer can see, and the chance
/// stream, are untouched. A card the viewer saw that the game still refers to (a Trainer card played this turn, the
/// source of a lasting effect, the last attack's card) keeps its identity and must be listed. Errors change nothing:
/// `NO_SUCH_PLAYER`, `UNKNOWN_CARD`, `UNSEEN_COUNT_MISMATCH`, `NAMED_CARD_NOT_LISTED`, `RECORD_NOT_TAKEN` (take the
/// record first) ([`Game::set_unseen`]).
pub fn set_unseen(g: &mut Game, viewer: usize, owner: usize, cards: &[DefId], seed: u32) -> R {
    g.set_unseen(viewer, owner, cards, seed)
}

/// The cards of `owner` that `viewer` cannot see, the ones [`set_unseen`] takes identities for ([`Game::unseen_cards`];
/// empty for a seat that doesn't exist).
pub fn unseen_cards(g: &Game, viewer: usize, owner: usize) -> Vec<CardId> {
    g.unseen_cards(viewer, owner)
}

/// Those of [`unseen_cards`] the game refers to by identity ([`Game::named_cards`]): [`set_unseen`] keeps their
/// identities, so its list must hold them.
pub fn unseen_kept(g: &Game, viewer: usize, owner: usize) -> Vec<CardId> {
    let named = g.named_cards();
    g.unseen_cards(viewer, owner).into_iter().filter(|c| named >> c & 1 != 0).collect()
}

/// What attack `attack` (an index into the attacks the Pokémon has as the Active Pokémon, granted ones included) of
/// `attacker` (a Pokémon in play of the player whose turn it is; on the Bench: as if it were the Active Pokémon) would do
/// now to the opposing Pokémon `target`: per class of coin sequences, the damage to each Pokémon, the Knock Outs, the
/// Prize cards taken, the questions answered on the way; at most `max_runs` evaluations ([`crate::what_if::attack`]). The
/// attack is used through the engine's own routine on copies, without the checks of using an attack (Energy, locks, the
/// first-turn rule, Special Conditions) and without the end of the turn; `g` is not changed.
pub fn attack_what_if(g: &Game, attacker: CardId, attack: usize, target: CardId, max_runs: usize) -> R<AttackWhatIf> {
    crate::what_if::attack(g, attacker, attack, target, &[], max_runs)
}

/// [`attack_what_if`] with the Energy cards `extra_energy` attached to the attacker on the copies before the attack, as if
/// already attached (no Attach event): each a card of that definition of the attacker's player, from their hand, else
/// their deck, else their discard pile; `AttackWhatIf::extra_attached` lists them with the zone each came from. Errors
/// as [`attack_what_if`], and `UNKNOWN_CARD`, `NOT_AN_ENERGY_CARD`, `NO_CARD_TO_ATTACH`, `CANNOT_BE_ATTACHED`.
pub fn attack_what_if_with_energy(g: &Game, attacker: CardId, attack: usize, target: CardId, extra_energy: &[DefId], max_runs: usize) -> R<AttackWhatIf> {
    crate::what_if::attack(g, attacker, attack, target, extra_energy, max_runs)
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

    /// Two deck cards of the other seat trade places through the facade; so does a hand card when a hand exists (the
    /// full hand cases are in search.rs); a card of two owners is refused.
    #[test]
    fn a_guess_is_re_identified_through_the_facade() {
        let d = deck();
        let mut g = start_game([&d, &d], 3, &StartOptions::default()).unwrap();
        let (a, b) = (g.st.players[1].deck.get(0).unwrap(), g.st.players[1].deck.get(1).unwrap());
        swap_hidden(&mut g, 0, a, b).unwrap();
        assert_eq!(locate_card(&g, a).unwrap().index, 1);
        if let Some(h) = g.st.players[1].hand.get(0) {
            swap_hidden(&mut g, 0, h, a).unwrap();
            assert_eq!(locate_card(&g, a).unwrap().zone, crate::place::Zone::Hand);
        }
        let mine = g.st.players[0].deck.get(0).unwrap();
        assert!(swap_hidden(&mut g, 0, mine, b).is_err());
    }

    /// A deck of the benchmark (`bench/spec.json`, by name).
    fn bench_deck(name: &str) -> Vec<DefId> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("bench/spec.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let d = v["decks"].as_array().unwrap().iter().find(|d| d["name"] == name).unwrap();
        d["cards"].as_array().unwrap().iter().map(|n| card_by_name(n.as_str().unwrap()).unwrap()).collect()
    }

    /// Games go on to the end after, at each of the first turn decisions, the other seat's unseen cards are given the
    /// identities of a third deck (keeping those the game refers to): random play, the invariants checked at every turn
    /// decision, no error, the game finishes.
    #[test]
    fn games_play_on_after_set_unseen() {
        let (a, b, third) = (bench_deck("meta-alakazam-powerful-hand"), bench_deck("meta-dragapult-ex"), bench_deck("meta-marnie-s-grimmsnarl-ex"));
        for seed in [4u32, 9, 23] {
            let mut g = start_game([&a, &b], seed, &StartOptions::default()).unwrap();
            let mut x = seed as u64 * 0x9E37_79B9_7F4A_7C15 + 1;
            let mut rnd = |n: usize| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) as usize % n
            };
            let mut turns = 0usize;
            let mut over = false;
            for _ in 0..20_000 {
                match next(&mut g).unwrap() {
                    Decision::Over { .. } => {
                        over = true;
                        break;
                    }
                    Decision::Turn { player, .. } => {
                        crate::invariants::check(&g).unwrap();
                        turns += 1;
                        let (v, o) = (player as usize, 1 - player as usize);
                        if turns <= 10 {
                            let hidden = unseen_cards(&g, v, o);
                            let kept = unseen_kept(&g, v, o);
                            let mut list: Vec<DefId> = kept.iter().map(|c| g.st.cards[*c as usize].def).collect();
                            list.extend(third.iter().cycle().skip(turns).take(hidden.len() - kept.len()));
                            set_unseen(&mut g, v, o, &list, seed + turns as u32).unwrap();
                            crate::invariants::check(&g).unwrap();
                        }
                        let Decision::Turn { actions, .. } = next(&mut g).unwrap() else { panic!("still the turn") };
                        let pass = actions.iter().position(|a| matches!(a, Action::Pass)).unwrap();
                        let k = if actions.len() == 1 || rnd(5) == 0 { pass } else { rnd(actions.len()) };
                        act(&mut g, actions[k]).unwrap();
                    }
                    Decision::Prompt { select, .. } => {
                        let mut picks = Vec::new();
                        loop {
                            let (mask, done) = pick_mask(&g, &select, &picks);
                            if picks.len() >= select.min_count && (picks.len() >= select.max_count || (done && rnd(3) == 0)) {
                                break;
                            }
                            let valid: Vec<usize> = (0..mask.len()).filter(|i| mask[*i]).collect();
                            if valid.is_empty() {
                                break;
                            }
                            picks.push(valid[rnd(valid.len())]);
                        }
                        answer(&mut g, &select, &picks).unwrap();
                    }
                }
            }
            assert!(over && turns > 10, "seed {seed}: the game did not finish ({turns} turns)");
        }
    }

    /// While the game record holds entries not taken, `set_unseen` is refused (they name card ids whose identity would
    /// change); once taken, it goes ahead.
    #[test]
    fn set_unseen_waits_for_the_record() {
        let d = deck();
        let mut g = start_game([&d, &d], 3, &StartOptions { record_events: true, ..Default::default() }).unwrap();
        let list: Vec<DefId> = unseen_cards(&g, 0, 1).iter().map(|c| g.st.cards[*c as usize].def).collect();
        assert_eq!(set_unseen(&mut g, 0, 1, &list, 1), Err(GameError("RECORD_NOT_TAKEN")));
        assert!(!take_record(&mut g).is_empty());
        set_unseen(&mut g, 0, 1, &list, 1).unwrap();
    }

    /// Abilities and card rules are told apart by `power_source`.
    #[test]
    fn a_power_names_its_source() {
        let fossil = cards().iter().find(|c| c.full_name == "Antique Root Fossil SCR 130").unwrap();
        assert_eq!(power_source(&fossil.powers[0]), RuleSource::CardRule);
    }
}
