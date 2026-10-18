//! The game record against played games and hand-built boards.

use super::*;
use crate::cause::CauseKind;
use crate::place::tests::game_of;
use crate::state::ListRef;

/// The benchmark's decks (bench/spec.json: the golden deck spec's random pool decks and meta decks).
fn decks() -> Vec<Vec<crate::carddb::DefId>> {
    let spec: serde_json::Value = serde_json::from_str(include_str!("../../bench/spec.json")).unwrap();
    spec["decks"].as_array().unwrap().iter().map(|d| crate::selfplay::expand_deck(&d["cards"]).unwrap()).collect()
}

/// Uniformly random play (the benchmark's policy) until the game ends or `max_turns`; returns the decisions made.
fn play(g: &mut Game, seed: u32, max_turns: i32) -> usize {
    let mut rng = crate::rng::Rng::new(seed.wrapping_mul(2654435761));
    let mut decisions = 0;
    for _ in 0..4000 {
        if g.st.turn > max_turns {
            break;
        }
        let Some(sel) = g.select().unwrap() else { break };
        decisions += 1;
        let mut picks: Vec<usize> = Vec::new();
        let mut stuck = false;
        loop {
            let (mask, stop) = g.pick_mask(&sel, &picks);
            let allowed: Vec<usize> = (0..sel.options.len()).filter(|j| mask[*j]).collect();
            let choices = allowed.len() + stop as usize;
            if choices == 0 {
                stuck = picks.is_empty();
                break;
            }
            let r = rng.index(choices);
            if r == allowed.len() {
                break;
            }
            picks.push(allowed[r]);
            if sel.max_count <= 1 {
                break;
            }
        }
        if stuck {
            let crate::game::Pending::Decision(pi) = g.pending() else { panic!("no option at a turn decision") };
            let pr = g.prompts.as_slice()[pi];
            let res = [serde_json::Value::Null, serde_json::json!([])].iter().find_map(|w| g.decode_answer(&pr, w).ok()).expect("an empty answer");
            g.resolve(pi, res).unwrap();
        } else {
            g.answer(&sel, &picks).unwrap();
        }
    }
    decisions
}

/// The zone a place is in, for the replay: the owner and the zone (any spot in play, any look).
fn zone(p: CardPlace) -> (u8, std::mem::Discriminant<Zone>) {
    (p.owner, std::mem::discriminant(&p.zone))
}

/// Kinds whose card entries are moves (a `from` and a `to`).
const MOVES: [EventKind; 16] = [
    EventKind::Discard,
    EventKind::PutIntoHand,
    EventKind::PutIntoDeck,
    EventKind::Draw,
    EventKind::LeavePlay,
    EventKind::TakePrizes,
    EventKind::EnterPlay,
    EventKind::Evolve,
    EventKind::Devolve,
    EventKind::Swap,
    EventKind::Attach,
    EventKind::MoveEnergy,
    EventKind::MoveTool,
    EventKind::Look,
    EventKind::SetPrizes,
    EventKind::ChangeActive,
];

/// A recorded game's entries replay its zones: every card is where the record's last move of it put it (a card the
/// record never moved is still in its owner's deck), every move has both places, no move is made outside an event; and
/// recording doesn't change the game (the same seed without the record ends in the same state).
#[test]
fn recorded_games_replay_their_zones() {
    let decks = decks();
    let mut kinds = std::collections::BTreeSet::new();
    for i in 0..decks.len() {
        let pair = [decks[i].as_slice(), decks[(i * 7 + 11) % decks.len()].as_slice()];
        let seed = 900 + i as u32;
        let mut g = Game::new(seed);
        g.record_on();
        g.start(pair).unwrap();
        let n = play(&mut g, seed, 60);
        let entries = g.take_record();
        assert_eq!(unattributed(), 0, "game {i}: a card moved with no event open");
        let mut plain = Game::new(seed);
        plain.start(pair).unwrap();
        assert_eq!(play(&mut plain, seed, 60), n, "game {i}: the record changed the play");
        assert_eq!(plain.state_hash(), g.state_hash(), "game {i}: the record changed the game");

        for (k, e) in entries.iter().enumerate() {
            kinds.insert(format!("{:?}", e.kind));
            assert_eq!(e.from.is_some(), e.to.is_some(), "game {i} entry {k}: {e:?}");
            if MOVES.contains(&e.kind) && e.card.is_some() {
                assert!(e.from.is_some(), "game {i} entry {k}: a move with no place: {e:?}");
            }
        }
        for c in 0..g.st.n_cards {
            let last = entries.iter().rev().find(|e| e.card == Some(c) && e.to.is_some()).and_then(|e| e.to);
            let now = g.locate_card(c).unwrap_or_else(|| panic!("game {i}: card {c} is nowhere"));
            match last {
                Some(to) => assert_eq!(zone(to), zone(now), "game {i}: card {c} is at {now:?}, the record's last move put it at {to:?}"),
                None => assert_eq!((now.owner as usize, now.zone), (g.st.owner(c), Zone::Deck), "game {i}: card {c} never moved but left the deck"),
            }
        }
    }
    // The games reach the common events.
    for k in ["Draw", "Shuffle", "Mulligan", "SetPrizes", "EnterPlay", "Reveal", "BeginTurn", "EndTurn", "Attach", "Discard", "PlayTrainer", "UseAttack", "Damage", "KnockOut", "LeavePlay", "TakePrizes", "ChangeActive", "CoinFlip", "Evolve"] {
        assert!(kinds.contains(k), "no {k} entry in the recorded games ({kinds:?})");
    }
}

const MON: &str = "Duraludon PRE 69";

fn board() -> Game {
    game_of(&[(MON, 4)], serde_json::json!({"me": {"reset": true, "active": MON, "hand": [MON, "Metal Energy MEE 8"], "discard": ["Metal Energy MEE 8"]}, "opp": {"reset": true, "active": MON}}))
}

/// Who sees a card that moves: whoever sees it where it was or where it goes (a draw its owner, a discard everyone, a card
/// set aside for a look the player looking, then its owner once in the hand).
#[test]
fn a_move_is_seen_where_the_card_is_seen() {
    let mut g = board();
    let me = g.st.active_player as usize;
    let mine = Cause::new(CauseKind::Trainer, None, me as u8);
    g.record_on();
    crate::engine::cards_zone::draw(&mut g, me, 1, mine).unwrap();
    let e = g.take_record();
    assert_eq!(e.len(), 1);
    assert_eq!((e[0].kind, e[0].from.unwrap().zone, e[0].to.unwrap().zone, e[0].seen_by), (EventKind::Draw, Zone::Deck, Zone::Hand, only(me as u8)));

    let card = g.st.players[me].hand.as_slice()[0];
    crate::engine::cards_zone::discard(&mut g, ListRef::Hand(me as u8), &[card], mine).unwrap();
    let e = g.take_record();
    assert_eq!((e[0].kind, e[0].card, e[0].to.unwrap().zone, e[0].seen_by), (EventKind::Discard, Some(card), Zone::Discard, ALL));

    // From the discard pile into the deck: public where it was.
    crate::engine::cards_zone::put_into_deck(&mut g, ListRef::Discard(me as u8), &[card], crate::spec::event::DeckPosition::Top, mine).unwrap();
    let e = g.take_record();
    let to = e.iter().find(|x| x.kind == EventKind::PutIntoDeck).unwrap();
    assert_eq!((to.to.unwrap().zone, to.to.unwrap().index, to.seen_by), (Zone::Deck, 0, ALL), "on top, seen by both");

    // The opponent looks at my top card: they see it, I don't; it comes back into my hand: I see it, they did already.
    let theirs = Cause::new(CauseKind::Trainer, None, 1 - me as u8);
    let top = g.st.players[me].deck.as_slice()[0];
    let t = g.alloc_temp(&[]);
    crate::engine::cards_zone::stage(&mut g, ListRef::Deck(me as u8), &[top], t, theirs);
    crate::engine::cards_zone::put_into_hand(&mut g, t, &[top], theirs).unwrap();
    let e = g.take_record();
    assert_eq!((e[0].kind, e[0].to.unwrap().zone, e[0].seen_by), (EventKind::Look, Zone::Looking { from: RulesZone::Deck }, only(1 - me as u8)));
    assert_eq!((e[1].kind, e[1].from.unwrap().zone, e[1].to.unwrap().zone, e[1].seen_by), (EventKind::PutIntoHand, Zone::Looking { from: RulesZone::Deck }, Zone::Hand, only(me as u8)));
}

/// The setup: the first-player coin, the shuffles, the opening draws seen by their owner, each hand's Basic check, the
/// Pokémon put down face down (seen by their owner), the Prize cards (seen by nobody), then all of them turned face up
/// (SSP rulebook p. 8 steps 2 to 8).
#[test]
fn the_setup_is_recorded_with_who_sees_it() {
    let d = decks().swap_remove(2);
    let mut g = Game::new(3);
    g.record_on();
    g.start([&d, &d]).unwrap();
    while let Some(sel) = g.select().unwrap() {
        if sel.select_type == crate::interface::SelectType::Main {
            break;
        }
        let mut picks = Vec::new();
        while picks.len() < sel.min_count {
            let (mask, _) = g.pick_mask(&sel, &picks);
            picks.push(mask.iter().position(|m| *m).unwrap());
        }
        g.answer(&sel, &picks).unwrap();
    }
    let e = g.take_record();
    assert!(matches!(e[0].detail, Detail::Coin { purpose: CoinPurpose::FirstPlayer, .. }));
    assert_eq!(e.iter().take_while(|x| x.kind != EventKind::Draw).filter(|x| x.kind == EventKind::Shuffle).count(), 2);
    let draws: Vec<_> = e.iter().filter(|x| x.kind == EventKind::Draw).collect();
    assert!(draws.len() >= 14);
    assert!(draws.iter().all(|x| x.seen_by == only(x.player)), "an opening draw is seen by its owner");
    for p in 0..2u8 {
        assert!(e.iter().any(|x| x.player == p && x.detail == (Detail::BasicCheck { found: true })), "player {p}'s hand has a Basic Pokémon");
    }
    let put: Vec<_> = e.iter().filter(|x| x.kind == EventKind::EnterPlay).collect();
    assert!(put.len() >= 2 && put.iter().all(|x| x.seen_by == only(x.player)), "face down: seen by the owner");
    let prizes: Vec<_> = e.iter().filter(|x| x.kind == EventKind::SetPrizes).collect();
    assert_eq!(prizes.len(), 12);
    assert!(prizes.iter().all(|x| x.seen_by == NOBODY && x.to.unwrap().zone == Zone::Prizes));
    for x in put {
        assert!(e.iter().any(|r| r.kind == EventKind::Reveal && r.card == x.card && r.seen_by == ALL), "turned face up at the end of the setup");
    }
    assert_eq!(e.last().map(|x| x.kind), Some(EventKind::Draw), "the first turn's draw ends the setup's record");
    assert!(e.iter().any(|x| x.kind == EventKind::BeginTurn));
}

/// Off by default; a legality trial (a fork) never records; a game restored from a copy keeps only the entries made up to
/// the copy (a rolled back action's go).
#[test]
fn off_by_default_trials_never_record_rollback_rewinds() {
    let mut g = board();
    assert!(!g.record.on());
    let me = g.st.active_player as usize;
    crate::engine::cards_zone::draw(&mut g, me, 1, Cause::new(CauseKind::Trainer, None, me as u8)).unwrap();
    g.record_on();
    assert!(g.take_record().is_empty(), "nothing recorded while off");
    let opts = crate::options::legal_actions(&g);
    assert!(!opts.is_empty());
    assert!(g.take_record().is_empty(), "the trials of the turn options record nothing");
    let backup = g.fork();
    assert!(!backup.record.on(), "a copy for a trial doesn't record");
    crate::engine::cards_zone::draw(&mut g, me, 2, Cause::new(CauseKind::Trainer, None, me as u8)).unwrap();
    g.copy_from(&backup);
    assert!(g.record.on(), "the game keeps recording");
    assert!(g.take_record().is_empty(), "the draws after the copy are gone with them");
    // A played Basic Pokémon: its EnterPlay from the hand, seen by both.
    let k = g.st.players[me].hand.iter().position(|c| g.st.cdef(c).name == "Duraludon").unwrap();
    let play = opts.iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
    g.act(play).unwrap();
    let e = g.take_record();
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!((e[0].kind, e[0].from.unwrap().zone, e[0].seen_by), (EventKind::EnterPlay, Zone::Hand, ALL));
    assert!(matches!(e[0].to.unwrap().zone, Zone::InPlay { spot: crate::place::Spot::Bench(_), role: crate::place::InPlayAs::Pokemon }));
}

/// A Special Condition put on a Pokémon that already has it the same way changes nothing and is not recorded (id1045,
/// id2014, id1778); a new one, or the same with another number of counters (id1044), is.
#[test]
fn a_condition_already_there_is_not_recorded_again() {
    use crate::types::SpecialCondition;
    let mut g = board();
    let opp = 1 - g.st.active_player as usize;
    let mine = Cause::new(CauseKind::Trainer, None, 1 - opp as u8);
    let t = crate::effects::SlotRef::new(opp, g.st.players[opp].active);
    g.record_on();
    assert!(crate::engine::condition::gain(&mut g, t, SpecialCondition::Confused, mine).unwrap());
    assert_eq!(g.take_record().iter().filter(|e| e.kind == EventKind::GainCondition).count(), 1);
    assert!(crate::engine::condition::gain(&mut g, t, SpecialCondition::Confused, mine).unwrap());
    assert_eq!(g.take_record().iter().filter(|e| e.kind == EventKind::GainCondition).count(), 0, "already Confused");
    let slot = g.st.slot(opp, t.s);
    assert!(crate::engine::condition::gain_changes(slot, SpecialCondition::Poisoned, 1));
    assert!(!crate::engine::condition::gain_changes(slot, SpecialCondition::Confused, 3));
    assert!(crate::engine::condition::gain_changes(slot, SpecialCondition::Confused, 4), "another number of counters");
    assert!(crate::engine::condition::gain_changes(slot, SpecialCondition::Asleep, 0), "Asleep replaces Confused");
}
