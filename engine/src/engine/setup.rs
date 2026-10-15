//! Game setup (APR G, standard setup): who begins, opening
//! hands, mulligans, starting Pokémon, prizes, mulligan draws.

use crate::carddb::{def, DefId};
use crate::effects::SlotRef;
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

/// Whether the deck is legal in the Standard format.
pub fn deck_is_valid(_st: &State, deck: &[DefId]) -> bool {
    if deck.len() != 60 {
        return false;
    }
    // Cards that count as the same name (data: `same_name_group`) can't share a deck under different names.
    for d in deck.iter().map(|d| def(*d)) {
        if d.same_name_group != 0 && deck.iter().any(|o| def(*o).name != d.name && def(*o).same_name_group == d.same_name_group) {
            return false;
        }
    }
    let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    let (mut ace, mut radiant, mut has_basic) = (false, false, false);
    for d in deck.iter().map(|d| def(*d)) {
        if d.is_pokemon() && d.stage == Stage2Basic::BASIC {
            has_basic = true;
        }
        if let Some(limit) = copy_limit(d) {
            let n = counts.entry(d.name).or_insert(0);
            *n += 1;
            if *n > limit.max(4) {
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

/// The deck-building limit of copies of a card's name: 4; 1 for an ACE SPEC or a Radiant Pokémon (one such card per
/// deck); none for a Basic Energy (`None`: any number). The deck check counts names against 4 and checks the one ACE SPEC /
/// Radiant per deck on its own; the rules that reason about how many copies can exist (a named search, user decision
/// 2026-10-13) read the limit here.
pub fn copy_limit(d: &crate::carddb::CardDef) -> Option<u32> {
    if d.is_energy() && d.energy_type == 0 {
        return None;
    }
    if d.has_tag(tag::ACE_SPEC) || d.has_tag(tag::RADIANT) {
        return Some(1);
    }
    Some(4)
}

struct Stage2Basic;
impl Stage2Basic {
    const BASIC: u8 = crate::types::Stage::Basic as u8;
}

fn is_starting_candidate(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.has_tag(tag::PLAY_DURING_SETUP) || (d.is_pokemon() && d.stage == Stage2Basic::BASIC)
}

/// Whether a card can be put on the Bench at setup: a Basic Pokémon, or a non-Pokémon card played during setup (Snorlax Doll, ruling 1478). A
/// Pokémon that only starts face down as the Active Pokémon (Cinderace's Explosiveness, ruling 1714) can't be Benched
/// (ruling 264; Advanced Rulebook I-G steps 6 and 7).
fn can_bench_at_setup(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    if d.is_pokemon() {
        d.stage == Stage2Basic::BASIC
    } else {
        d.has_tag(tag::PLAY_DURING_SETUP)
    }
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
    g.prompt(p, PromptPurpose::Activate, PromptKind::Confirm, Cont::Setup(f));
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
        // Both players mulligan (APR G): each hand goes back into its deck (the Mulligan routine, user decision D2).
        crate::engine::cards_zone::mulligan(g, 0);
        crate::engine::cards_zone::mulligan(g, 1);
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
    // The coin flip for who goes first (APR G).
    let frame = SetupFrame { stage: Stage::Coin, who_begins: false, pm: 0, om: 0, php: false, ohp: false, branch: 0, shown: 0 };
    g.prompt(0, PromptPurpose::FirstPlayerFlip, PromptKind::CoinFlip, Cont::Setup(frame));
    Ok(())
}

fn deal(g: &mut Game, mut f: SetupFrame) -> R {
    f.stage = Stage::Dealt;
    g.prompt_group(&[(0, PromptPurpose::Shuffle, PromptKind::ShuffleDeck), (1, PromptPurpose::Shuffle, PromptKind::ShuffleDeck)], Cont::Setup(f));
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
    g.prompt(
        p,
        PromptPurpose::SetupActive,
        PromptKind::ChooseCards { cards: ListRef::Hand(p as u8), filter: Filter::none(), opts },
        Cont::Setup(f),
    );
}

fn put_starting_pokemons_and_prizes(g: &mut Game, p: usize, cards: &[CardId]) -> R {
    if cards.is_empty() {
        return Ok(());
    }
    let active = g.st.players[p].active;
    crate::engine::enter::put_at_setup(g, cards[0], SlotRef::new(p, active))?;
    // A card that can't be Benched stays in hand.
    let mut benched = 0;
    for &c in cards.iter().skip(1) {
        if !can_bench_at_setup(g, c) {
            continue;
        }
        let b = g.st.players[p].bench.as_slice()[benched];
        crate::engine::enter::put_at_setup(g, c, SlotRef::new(p, b))?;
        benched += 1;
    }
    crate::engine::cards_zone::set_prizes(g, p);
    Ok(())
}

/// The rule's draws of the setup (APR G: 7 cards; a mulligan's new hand; the cards drawn for the opponent's mulligans): a
/// Draw event by the rule.
fn setup_draw(g: &mut Game, p: usize, n: usize) -> R {
    crate::engine::cards_zone::draw(g, p, n, crate::cause::Cause::rule(crate::cause::RuleWhich::Setup, p as u8))?;
    Ok(())
}

fn mulligan_shuffle(g: &mut Game, p: usize, mut f: SetupFrame, next: Stage) {
    crate::engine::cards_zone::mulligan(g, p);
    f.stage = next;
    g.prompt(p, PromptPurpose::Shuffle, PromptKind::ShuffleDeck, Cont::Setup(f));
}

/// Number of show-mulligan prompts for the frame's branch, in order.
fn mulligan_prompt_owners(f: &SetupFrame) -> Vec<(usize, PromptPurpose)> {
    let (pm, om) = (f.pm > 0, f.om > 0);
    let mut v = Vec::new();
    match f.branch {
        0 => {
            if om {
                v.push((0, PromptPurpose::MulliganOpponents));
                v.push((1, PromptPurpose::MulliganYours));
            }
            if pm {
                v.push((0, PromptPurpose::MulliganYours));
                v.push((1, PromptPurpose::MulliganOpponents));
            }
        }
        1 => {
            if pm {
                v.push((1, PromptPurpose::MulliganOpponents));
                v.push((0, PromptPurpose::MulliganYours));
            }
            if om {
                v.push((1, PromptPurpose::MulliganYours));
                v.push((0, PromptPurpose::MulliganOpponents));
            }
        }
        _ => {
            if pm || om {
                if pm {
                    v.push((0, PromptPurpose::MulliganYours));
                    v.push((1, PromptPurpose::MulliganOpponents));
                }
                if om {
                    v.push((1, PromptPurpose::MulliganYours));
                    v.push((0, PromptPurpose::MulliganOpponents));
                }
            }
        }
    }
    v
}

fn show_mulligans(g: &mut Game, mut f: SetupFrame) -> R {
    let owners = mulligan_prompt_owners(&f);
    if (f.shown as usize) < owners.len() {
        let (p, purpose) = owners[f.shown as usize];
        f.shown += 1;
        f.stage = Stage::ShowMulligans;
        g.prompt(p, purpose, PromptKind::ShowMulligan, Cont::Setup(f));
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
    g.prompt(
        drawer,
        PromptPurpose::DrawCount,
        PromptKind::Select { values: SelectValues::DrawCards(extra), allow_cancel: false, default_value: 0 },
        Cont::Setup(f),
    );
    Ok(())
}

fn allow_extra_bench_placement(g: &mut Game, p: usize, mut f: SetupFrame) -> R {
    let pl = &g.st.players[p];
    let mut in_play: Vec<CardId> = Vec::new();
    for &b in pl.bench.iter() {
        in_play.extend(pl.slots[b as usize].cards().iter());
    }
    in_play.extend(pl.slots[pl.active as usize].cards().iter());
    let hand: Vec<CardId> = pl.hand.iter().collect();
    let new_basics: Vec<usize> =
        hand.iter().enumerate().filter(|(_, c)| is_starting_candidate(g, **c) && can_bench_at_setup(g, **c) && !in_play.contains(c)).map(|(i, _)| i).collect();
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
    g.prompt(
        p,
        PromptPurpose::SetupActive,
        PromptKind::ChooseCards { cards: ListRef::Hand(p as u8), filter: Filter::none(), opts },
        Cont::Setup(f),
    );
    Ok(())
}

pub fn resume(g: &mut Game, mut f: SetupFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        Stage::Coin => {
            // The flip for who goes first (player 1 flips).
            crate::engine::condition::coin_flipped(g, 0, crate::spec::event::CoinPurpose::FirstPlayer, first.as_bool(), crate::cause::Cause::rule(crate::cause::RuleWhich::Setup, 0))?;
            f.who_begins = first.as_bool();
            f.stage = Stage::GoFirst;
            g.prompt(if f.who_begins { 0 } else { 1 }, PromptPurpose::ChooseFirstPlayer, PromptKind::Confirm, Cont::Setup(f));
            Ok(())
        }
        Stage::GoFirst => {
            let choice = matches!(first, Res::Bool(true));
            g.st.active_player = if choice == f.who_begins { 0 } else { 1 };
            g.st.first_player = g.st.active_player;
            g.st.bump();
            deal(g, f)
        }
        Stage::Dealt => {
            for (p, r) in results.iter().enumerate().take(2) {
                if let Res::Order(o) = r {
                    crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
                    g.st.bump_layout();
                }
            }
            for p in 0..2 {
                setup_draw(g, p, 7)?;
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
            put_starting_pokemons_and_prizes(g, me, first.cards())?;
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
                g.st.bump_layout();
            }
            setup_draw(g, other, 7)?;
            if needs_starter_confirm(g, other) {
                starter_confirm(g, other, f, if other == 1 { Stage::ConfA } else { Stage::ConfB });
                return Ok(());
            }
            after_mulligan_draw(g, other, f, hand_has_starting_pokemon(g, other))
        }
        Stage::AOppSetup | Stage::BPlayerSetup => {
            let other = if f.stage == Stage::AOppSetup { 1 } else { 0 };
            put_starting_pokemons_and_prizes(g, other, first.cards())?;
            f.shown = 0;
            show_mulligans(g, f)
        }
        Stage::CFirst => {
            put_starting_pokemons_and_prizes(g, 0, first.cards())?;
            choose_starting(g, 1, f, Stage::CSecond);
            Ok(())
        }
        Stage::CSecond => {
            put_starting_pokemons_and_prizes(g, 1, first.cards())?;
            f.shown = 0;
            show_mulligans(g, f)
        }
        Stage::ShowMulligans => show_mulligans(g, f),
        Stage::ExtraDraw => {
            let drawer = if f.branch == 0 { 0 } else { 1 };
            let extra = if f.branch == 0 { f.om - f.pm } else { f.pm - f.om };
            let choice = first.as_int() as u8;
            let n = extra.saturating_sub(choice);
            setup_draw(g, drawer, n as usize)?;
            allow_extra_bench_placement(g, drawer, f)
        }
        Stage::ExtraBench => {
            let drawer = if f.branch == 0 { 0 } else { 1 };
            for &c in first.cards() {
                let pl = &g.st.players[drawer];
                let empty = pl.bench.iter().copied().find(|b| pl.slots[*b as usize].cards().is_empty());
                if let Some(b) = empty {
                    crate::engine::enter::put_at_setup(g, c, crate::effects::SlotRef::new(drawer, b))?;
                }
            }
            finish(g)
        }
    }
}

fn finish(g: &mut Game) -> R {
    let first = g.st.active_player as usize;
    let second = 1 - first;
    // The Pokémon put down at setup count as entering play on their owner's first turn (`Slot::entered_turn`).
    for s in g.st.players[first].in_play() {
        g.st.players[first].slots[s as usize].entered_turn = 1;
    }
    for s in g.st.players[second].in_play() {
        g.st.players[second].slots[s as usize].entered_turn = 2;
    }
    // Both Active Pokémon take hold together: the player who goes first is stamped first.
    crate::spec::passive::lock_sync(g);
    crate::engine::phase::init_next_turn(g)
}

#[cfg(test)]
mod same_name_tests {
    use super::*;
    use crate::carddb::def_by_full_name;

    /// The replaced hard-coded rule (frozen copy): the old same-name check, then the unchanged rest of the rule.
    fn old_deck_is_valid(st: &State, deck: &[DefId]) -> bool {
        if deck.len() == 60 {
            let names: Vec<&str> = deck.iter().map(|d| def(*d).name).collect();
            let has = |n: &str| names.contains(&n);
            if (has("Professor Sycamore") && has("Professor Juniper"))
                || (has("Professor Juniper") && has("Professor's Research"))
                || (has("Professor Sycamore") && has("Professor's Research"))
                || (has("Lysandre") && has("Boss's Orders"))
            {
                return false;
            }
        }
        rest_of_the_rule(st, deck)
    }

    /// `deck_is_valid` as it was after the same-name check.
    fn rest_of_the_rule(_st: &State, deck: &[DefId]) -> bool {
        if deck.len() != 60 {
            return false;
        }
        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        let (mut ace, mut radiant, mut has_basic) = (false, false, false);
        for d in deck.iter().map(|d| def(*d)) {
            if d.is_pokemon() && d.stage == Stage2Basic::BASIC {
                has_basic = true;
            }
            if let Some(limit) = copy_limit(d) {
                let n = counts.entry(d.name).or_insert(0);
                *n += 1;
                if *n > limit.max(4) {
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

    /// None when the deck names a card outside the card data (some meta decks do).
    fn load(path: &std::path::Path) -> Option<Vec<DefId>> {
        let mut deck = Vec::new();
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (n, key) = line.split_once(' ').unwrap();
            let id = def_by_full_name(key)?;
            for _ in 0..n.parse::<usize>().unwrap() {
                deck.push(id);
            }
        }
        Some(deck)
    }

    #[test]
    fn same_name_groups_agree_with_the_old_rule() {
        let st = State::new();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../decks");
        let mut files = Vec::new();
        for dir in [root.clone(), root.join("meta")] {
            for e in std::fs::read_dir(&dir).unwrap() {
                let p = e.unwrap().path();
                if p.extension().map_or(false, |x| x == "txt") {
                    files.push(p);
                }
            }
        }
        assert!(files.len() >= 16);
        let boss: Vec<DefId> = ["Boss's Orders ASC 183", "Boss's Orders PAL 172"].iter().map(|k| def_by_full_name(k).unwrap()).collect();
        let mut compared = 0;
        for f in &files {
            let Some(deck) = load(f) else { continue };
            assert_eq!(old_deck_is_valid(&st, &deck), deck_is_valid(&st, &deck), "{}", f.display());
            compared += 1;
            // Constructed decks mixing the two printings of Boss's Orders (same name) in place of other cards.
            for k in 0..=5 {
                for i in 0..2usize {
                    let mut d = deck.clone();
                    for j in 0..k.min(d.len()) {
                        d[j] = boss[(i + j) % 2];
                    }
                    assert_eq!(old_deck_is_valid(&st, &d), deck_is_valid(&st, &d), "{} k={}", f.display(), k);
                    compared += 1;
                }
            }
        }
        assert!(compared >= 16 * 11, "{}", compared);
    }

    /// Every group is complete on the cards that exist: all defs of one name (every printing) carry the same group.
    #[test]
    fn groups_are_symmetric() {
        let all = crate::gen::cards::CARDS;
        assert!(all.iter().any(|d| d.same_name_group != 0));
        for d in all.iter() {
            for o in all.iter().filter(|o| o.name == d.name) {
                assert_eq!(o.same_name_group, d.same_name_group, "{} vs {}", d.full_name, o.full_name);
            }
        }
    }
}
