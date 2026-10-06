//! Game setup (`setup-reducer.ts`, standard setup): who begins, opening
//! hands, mulligans, starting Pokémon, prizes, mulligan draws.

use crate::carddb::{def, DefId};
use crate::effects::Effect;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Coin,
    GoFirst,
    Dealt,
    /// Branch A (player has Basic, opponent mulligans): after player's setup.
    AStart,
    AMulliganed,
    AOppSetup,
    /// Branch B (opponent has Basic, player mulligans).
    BStart,
    BMulliganed,
    BPlayerSetup,
    /// Branch C (both have Basics).
    CFirst,
    CSecond,
    /// Show-mulligan info prompts; `shown` counts those already resolved.
    ShowMulligans,
    /// Explosiveness confirm: the hand's only starter is Cinderace (ruling 1714); player 0 / player 1 after the deal,
    /// and the mulligan loops of branch A (player 1) and B (player 0).
    ConfP0,
    ConfP1,
    ConfA,
    ConfB,
    ExtraDraw,
    ExtraBench,
}

#[derive(Clone, Copy, Debug)]
pub struct SetupFrame {
    pub stage: Stage,
    pub who_begins: bool,
    pub pm: u8,
    pub om: u8,
    pub php: bool,
    pub ohp: bool,
    /// Which branch: 0 = A, 1 = B, 2 = C.
    pub branch: u8,
    pub shown: u8,
}

/// `DeckAnalyser.isValid` (standard format).
pub fn deck_is_valid(_st: &State, deck: &[DefId]) -> bool {
    if deck.len() != 60 {
        return false;
    }
    let names: Vec<&str> = deck.iter().map(|d| def(*d).name).collect();
    let has = |n: &str| names.contains(&n);
    if (has("Professor Sycamore") && has("Professor Juniper"))
        || (has("Professor Juniper") && has("Professor's Research"))
        || (has("Professor Sycamore") && has("Professor's Research"))
        || (has("Lysandre") && has("Boss's Orders"))
    {
        return false;
    }
    let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    let (mut ace, mut radiant, mut has_basic) = (false, false, false);
    for d in deck.iter().map(|d| def(*d)) {
        if d.is_pokemon() && d.stage == Stage2Basic::BASIC {
            has_basic = true;
        }
        if !(d.is_energy() && d.energy_type == 0) {
            let n = counts.entry(d.name).or_insert(0);
            *n += 1;
            if *n > 4 {
                return false;
            }
        }
        if d.has_tag(tag::ACE_SPEC) {
            if ace {
                return false;
            }
            ace = true;
        }
        if d.has_tag(tag::RADIANT) {
            if radiant {
                return false;
            }
            radiant = true;
        }
    }
    has_basic
}

struct Stage2Basic;
impl Stage2Basic {
    const BASIC: u8 = crate::types::Stage::Basic as u8;
}

fn is_starting_candidate(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.has_tag(tag::PLAY_DURING_SETUP) || (d.is_pokemon() && d.stage == Stage2Basic::BASIC)
}

fn hand_has_starting_pokemon(g: &Game, p: usize) -> bool {
    g.st.players[p].hand.iter().any(|c| {
        let d = g.st.cdef(c);
        (d.is_pokemon() && d.stage == Stage2Basic::BASIC) || d.has_tag(tag::PLAY_DURING_SETUP)
    })
}

/// A hand with no Basic Pokémon whose only starter is a setup card (Cinderace's Explosiveness): the player
/// chooses between putting it Active and a mulligan (ruling 1714).
fn needs_starter_confirm(g: &Game, p: usize) -> bool {
    hand_has_starting_pokemon(g, p) && !g.st.players[p].hand.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_pokemon() && d.stage == Stage2Basic::BASIC
    })
}

fn starter_confirm(g: &mut Game, p: usize, mut f: SetupFrame, stage: Stage) {
    f.stage = stage;
    let id = g.player_id(p);
    g.prompt(id, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::Setup(f));
}

/// Both hands are dealt: ask player 0 (when needed), then player 1.
fn eval_p0(g: &mut Game, mut f: SetupFrame) -> R {
    if needs_starter_confirm(g, 0) {
        starter_confirm(g, 0, f, Stage::ConfP0);
        return Ok(());
    }
    f.php = hand_has_starting_pokemon(g, 0);
    eval_p1(g, f)
}

fn eval_p1(g: &mut Game, mut f: SetupFrame) -> R {
    if needs_starter_confirm(g, 1) {
        starter_confirm(g, 1, f, Stage::ConfP1);
        return Ok(());
    }
    f.ohp = hand_has_starting_pokemon(g, 1);
    after_eval(g, f)
}

fn after_eval(g: &mut Game, mut f: SetupFrame) -> R {
    if !f.php && !f.ohp {
        f.pm += 1;
        f.om += 1;
        g.move_to(ListRef::Hand(0), ListRef::Deck(0), None);
        g.move_to(ListRef::Hand(1), ListRef::Deck(1), None);
        return deal(g, f);
    }
    if f.php && !f.ohp {
        f.branch = 0;
        choose_starting(g, 0, f, Stage::AStart);
    } else if !f.php && f.ohp {
        f.branch = 1;
        choose_starting(g, 1, f, Stage::BStart);
    } else {
        f.branch = 2;
        choose_starting(g, 0, f, Stage::CFirst);
    }
    Ok(())
}

/// The mulliganing player of branch A (`other` = 1) or B (0) drew 7: `has` says whether the hand has a starter.
fn after_mulligan_draw(g: &mut Game, other: usize, mut f: SetupFrame, has: bool) -> R {
    let a = other == 1;
    if has {
        let next = if a { Stage::AOppSetup } else { Stage::BPlayerSetup };
        choose_starting(g, other, f, next);
    } else if a {
        f.om += 1;
        mulligan_shuffle(g, other, f, Stage::AMulliganed);
    } else {
        f.pm += 1;
        mulligan_shuffle(g, other, f, Stage::BMulliganed);
    }
    Ok(())
}

pub fn start(g: &mut Game) -> R {
    let (who, _) = g.run_fx(Effect::WhoBegins { player: None })?;
    let frame = SetupFrame { stage: Stage::Coin, who_begins: false, pm: 0, om: 0, php: false, ohp: false, branch: 0, shown: 0 };
    if let Effect::WhoBegins { player: Some(p) } = who {
        g.st.active_player = p;
        return deal(g, frame);
    }
    let id = g.player_id(0);
    g.prompt(id, "SETUP_WHO_BEGINS_FLIP", PromptKind::CoinFlip, Cont::Setup(frame));
    Ok(())
}

fn deal(g: &mut Game, mut f: SetupFrame) -> R {
    f.stage = Stage::Dealt;
    let (a, b) = (g.player_id(0), g.player_id(1));
    g.prompt_group(&[(a, "", PromptKind::ShuffleDeck), (b, "", PromptKind::ShuffleDeck)], Cont::Setup(f));
    Ok(())
}

fn choose_starting(g: &mut Game, p: usize, mut f: SetupFrame, next: Stage) {
    let mut opts = ChooseCardsOpts::new(1, 6, false);
    let hand: Vec<CardId> = g.st.players[p].hand.iter().collect();
    for (i, &c) in hand.iter().enumerate() {
        if !is_starting_candidate(g, c) {
            opts.blocked.push(i as u8);
        }
    }
    f.stage = next;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_STARTING_POKEMONS",
        PromptKind::ChooseCards { cards: ListRef::Hand(p as u8), filter: Filter::none(), opts },
        Cont::Setup(f),
    );
}

fn put_starting_pokemons_and_prizes(g: &mut Game, p: usize, cards: &[CardId]) {
    if cards.is_empty() {
        return;
    }
    let pl = p as u8;
    let active = g.st.players[p].active;
    g.move_card_to(ListRef::Hand(pl), cards[0], ListRef::Slot(pl, active));
    for (i, &c) in cards.iter().enumerate().skip(1) {
        let b = g.st.players[p].bench.as_slice()[i - 1];
        g.move_card_to(ListRef::Hand(pl), c, ListRef::Slot(pl, b));
    }
    for i in 0..6u8 {
        g.move_to(ListRef::Deck(pl), ListRef::Prize(pl, i), Some(1));
    }
}

fn mulligan_shuffle(g: &mut Game, p: usize, mut f: SetupFrame, next: Stage) {
    g.move_to(ListRef::Hand(p as u8), ListRef::Deck(p as u8), None);
    f.stage = next;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Setup(f));
}

/// Number of show-mulligan prompts for the frame's branch, in order.
fn mulligan_prompt_owners(f: &SetupFrame) -> Vec<(usize, &'static str)> {
    let (pm, om) = (f.pm > 0, f.om > 0);
    let mut v = Vec::new();
    match f.branch {
        0 => {
            if om {
                v.push((0, "SETUP_OPPONENT_NO_BASIC"));
                v.push((1, "SETUP_PLAYER_NO_BASIC"));
            }
            if pm {
                v.push((0, "SETUP_PLAYER_NO_BASIC"));
                v.push((1, "SETUP_OPPONENT_NO_BASIC"));
            }
        }
        1 => {
            if pm {
                v.push((1, "SETUP_OPPONENT_NO_BASIC"));
                v.push((0, "SETUP_PLAYER_NO_BASIC"));
            }
            if om {
                v.push((1, "SETUP_PLAYER_NO_BASIC"));
                v.push((0, "SETUP_OPPONENT_NO_BASIC"));
            }
        }
        _ => {
            if pm || om {
                if pm {
                    v.push((0, "SETUP_PLAYER_NO_BASIC"));
                    v.push((1, "SETUP_OPPONENT_NO_BASIC"));
                }
                if om {
                    v.push((1, "SETUP_PLAYER_NO_BASIC"));
                    v.push((0, "SETUP_OPPONENT_NO_BASIC"));
                }
            }
        }
    }
    v
}

fn show_mulligans(g: &mut Game, mut f: SetupFrame) -> R {
    let owners = mulligan_prompt_owners(&f);
    if (f.shown as usize) < owners.len() {
        let (p, msg) = owners[f.shown as usize];
        f.shown += 1;
        f.stage = Stage::ShowMulligans;
        let id = g.player_id(p);
        g.prompt(id, msg, PromptKind::ShowMulligan, Cont::Setup(f));
        return Ok(());
    }
    after_mulligans(g, f)
}

fn after_mulligans(g: &mut Game, mut f: SetupFrame) -> R {
    // The player who had a Basic first may draw the mulligan difference.
    let (drawer, extra) = match f.branch {
        0 => (0usize, f.om.saturating_sub(f.pm)),
        1 => (1usize, f.pm.saturating_sub(f.om)),
        _ => return finish(g),
    };
    if extra == 0 {
        return finish(g);
    }
    f.stage = Stage::ExtraDraw;
    let id = g.player_id(drawer);
    g.prompt(
        id,
        "WANT_TO_DRAW_CARDS",
        PromptKind::Select { values: SelectValues::DrawCards(extra), allow_cancel: false, default_value: 0 },
        Cont::Setup(f),
    );
    Ok(())
}

fn allow_extra_bench_placement(g: &mut Game, p: usize, mut f: SetupFrame) -> R {
    let pl = &g.st.players[p];
    let mut in_play: Vec<CardId> = Vec::new();
    for &b in pl.bench.iter() {
        in_play.extend(pl.slots[b as usize].cards.iter());
    }
    in_play.extend(pl.slots[pl.active as usize].cards.iter());
    let hand: Vec<CardId> = pl.hand.iter().collect();
    let new_basics: Vec<usize> =
        hand.iter().enumerate().filter(|(_, c)| is_starting_candidate(g, **c) && !in_play.contains(c)).map(|(i, _)| i).collect();
    if new_basics.is_empty() {
        return finish(g);
    }
    let mut opts = ChooseCardsOpts::new(0, new_basics.len() as u8, false);
    for i in 0..hand.len() {
        if !new_basics.contains(&i) {
            opts.blocked.push(i as u8);
        }
    }
    f.stage = Stage::ExtraBench;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_STARTING_POKEMONS",
        PromptKind::ChooseCards { cards: ListRef::Hand(p as u8), filter: Filter::none(), opts },
        Cont::Setup(f),
    );
    Ok(())
}

pub fn resume(g: &mut Game, mut f: SetupFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        Stage::Coin => {
            f.who_begins = first.as_bool();
            f.stage = Stage::GoFirst;
            let id = if f.who_begins { g.player_id(0) } else { g.player_id(1) };
            g.prompt(id, "GO_FIRST", PromptKind::Confirm, Cont::Setup(f));
            Ok(())
        }
        Stage::GoFirst => {
            let choice = matches!(first, Res::Bool(true));
            g.st.active_player = if choice == f.who_begins { 0 } else { 1 };
            deal(g, f)
        }
        Stage::Dealt => {
            for (p, r) in results.iter().enumerate().take(2) {
                if let Res::Order(o) = r {
                    crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
                }
            }
            for p in 0..2u8 {
                g.move_to(ListRef::Deck(p), ListRef::Hand(p), Some(7));
            }
            eval_p0(g, f)
        }
        Stage::ConfP0 => {
            f.php = matches!(first, Res::Bool(true));
            eval_p1(g, f)
        }
        Stage::ConfP1 => {
            f.ohp = matches!(first, Res::Bool(true));
            after_eval(g, f)
        }
        Stage::ConfA => after_mulligan_draw(g, 1, f, matches!(first, Res::Bool(true))),
        Stage::ConfB => after_mulligan_draw(g, 0, f, matches!(first, Res::Bool(true))),
        Stage::AStart | Stage::BStart => {
            let (me, other) = if f.stage == Stage::AStart { (0, 1) } else { (1, 0) };
            put_starting_pokemons_and_prizes(g, me, first.cards());
            // The other player mulligans until they have a Basic.
            if f.stage == Stage::AStart {
                f.om += 1;
                mulligan_shuffle(g, other, f, Stage::AMulliganed);
            } else {
                f.pm += 1;
                mulligan_shuffle(g, other, f, Stage::BMulliganed);
            }
            Ok(())
        }
        Stage::AMulliganed | Stage::BMulliganed => {
            let other = if f.stage == Stage::AMulliganed { 1 } else { 0 };
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[other].deck, o.as_slice());
            }
            g.move_to(ListRef::Deck(other as u8), ListRef::Hand(other as u8), Some(7));
            if needs_starter_confirm(g, other) {
                starter_confirm(g, other, f, if other == 1 { Stage::ConfA } else { Stage::ConfB });
                return Ok(());
            }
            after_mulligan_draw(g, other, f, hand_has_starting_pokemon(g, other))
        }
        Stage::AOppSetup | Stage::BPlayerSetup => {
            let other = if f.stage == Stage::AOppSetup { 1 } else { 0 };
            put_starting_pokemons_and_prizes(g, other, first.cards());
            f.shown = 0;
            show_mulligans(g, f)
        }
        Stage::CFirst => {
            put_starting_pokemons_and_prizes(g, 0, first.cards());
            choose_starting(g, 1, f, Stage::CSecond);
            Ok(())
        }
        Stage::CSecond => {
            put_starting_pokemons_and_prizes(g, 1, first.cards());
            f.shown = 0;
            show_mulligans(g, f)
        }
        Stage::ShowMulligans => show_mulligans(g, f),
        Stage::ExtraDraw => {
            let drawer = if f.branch == 0 { 0 } else { 1 };
            let extra = if f.branch == 0 { f.om - f.pm } else { f.pm - f.om };
            let choice = first.as_int() as u8;
            let n = extra.saturating_sub(choice);
            g.move_to(ListRef::Deck(drawer as u8), ListRef::Hand(drawer as u8), Some(n as usize));
            allow_extra_bench_placement(g, drawer, f)
        }
        Stage::ExtraBench => {
            let drawer = if f.branch == 0 { 0 } else { 1 };
            for &c in first.cards() {
                let pl = &g.st.players[drawer];
                let empty = pl.bench.iter().copied().find(|b| pl.slots[*b as usize].cards.is_empty());
                if let Some(b) = empty {
                    g.move_card_to(ListRef::Hand(drawer as u8), c, ListRef::Slot(drawer as u8, b));
                }
            }
            finish(g)
        }
    }
}

fn finish(g: &mut Game) -> R {
    let first = g.st.active_player as usize;
    let second = 1 - first;
    for s in g.st.players[first].in_play().iter() {
        g.st.players[first].slots[*s as usize].pokemon_played_turn = 1;
    }
    for s in g.st.players[second].in_play().iter() {
        g.st.players[second].slots[*s as usize].pokemon_played_turn = 2;
    }
    crate::engine::game_effect::stamp_starting_ability_locks(g);
    crate::engine::phase::init_next_turn(g)
}
