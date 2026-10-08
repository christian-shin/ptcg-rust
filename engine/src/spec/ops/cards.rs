//! Moving cards between zones, Energy, Prizes (vocabulary v1 "Operations",
//! cards and zones).
//!
//! Chosen cards travel in card registers (`Frame::cards`, scratch lists):
//! `Pick` and `Search` fill register `into`, `Move` can fill one, and later
//! steps read them (`CardSel::Chosen`, `Cond::Chosen`, `Num::RegCount`).
//! Attack choices whose options exist before the damage are made at step D
//! (`choice` / `resume_choice`) and recorded as card ids.

use super::super::run::{Flow, Frame, CHOICE_NONE, CHOICE_YES, NONE};
use super::super::*;
use crate::effects::{AtkBase, Effect, SlotRef};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

// ---------------------------------------------------------------------------
// Records

/// Move cards from one zone to another.
pub struct MoveSpec {
    pub from: ZoneRef,
    pub to: ZoneRef,
    pub cards: CardSel,
    pub place: Place,
    /// Show the moved cards to this player before they move.
    pub reveal: Option<Who>,
    /// Register that receives the cards moved.
    pub into: Option<u8>,
    /// Shuffle the selected cards first, with the game RNG ("they shuffle their hand").
    pub shuffle_first: bool,
}
impl MoveSpec {
    pub const DEFAULT: MoveSpec = MoveSpec {
        from: ZoneRef(Who::Me, Zone::Hand),
        to: ZoneRef(Who::Me, Zone::Discard),
        cards: CardSel::All,
        place: Place::End,
        reveal: None,
        into: None,
        shuffle_first: false,
    };
}
pub enum CardSel {
    All,
    /// The first cards of the zone (the top of a deck).
    Top(Num),
    /// The last cards of the zone (the bottom of a deck).
    Bottom(Num),
    /// Random cards (game RNG, without replacement).
    Random(Num),
    /// The cards of a register.
    Chosen(u8),
    /// The Pokémon Tools attached to the Pokémon (`from` is ignored); one move per Tool.
    Tools(SlotExpr),
    // --- S3-4 appends ---
    /// The Stadium in play, wherever it is (`from` is ignored); it goes to its owner's discard pile.
    Stadium,
    /// The first card of the zone matching the predicate (the resolving card never counts); nothing is asked.
    First(Pred),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    /// Appended to the destination (the bottom of a deck).
    End,
    Top,
}

/// Choose cards of a zone without moving them.
pub struct PickSpec {
    pub chooser: Who,
    pub from: ZoneRef,
    pub predicate: Pred,
    pub bounds: Bounds,
    /// Register that receives the chosen cards.
    pub into: u8,
    pub cancel: bool,
    /// At most this many chosen cards of a kind.
    pub caps: &'static [Cap],
    /// The chosen cards must differ in type (`different_types`).
    pub distinct_types: bool,
    /// Prompt message (empty: a generic one).
    pub msg: &'static str,
    /// Nothing to choose does not make a Trainer unplayable (another step still has an effect).
    pub soft: bool,
}
impl PickSpec {
    pub const DEFAULT: PickSpec = PickSpec {
        chooser: Who::Me,
        from: ZoneRef(Who::Me, Zone::Deck),
        predicate: Pred::Any,
        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
        into: 0,
        cancel: false,
        caps: &[],
        distinct_types: false,
        msg: "",
        soft: false,
    };
}
pub struct Cap {
    pub kind: CapKind,
    pub max: Num,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapKind {
    Pokemon,
    BasicEnergy,
    Energy,
    Trainer,
    Tool,
    Item,
    Stadium,
    Supporter,
}

pub struct DrawSpec {
    pub who: Who,
    pub amount: DrawAmount,
}
pub enum DrawAmount {
    Count(Num),
    /// Draw until the hand has this many cards (no draw if it has as many).
    UntilHandSize(Num),
}
pub struct ShuffleSpec {
    pub zone: ZoneRef,
}
/// Show cards to a player (an information screen, not a decision).
pub struct RevealSpec {
    pub cards: RevealWhat,
    pub to: Who,
    /// Show even when there are no cards (the card text says "reveals their hand").
    pub when_empty: bool,
}
pub enum RevealWhat {
    Zone(ZoneRef),
    Chosen(u8),
}
/// Search a zone for cards matching `pick` and put them somewhere. An attack
/// whose search can't be carried out still resolves (rulings 336, 337, 1790);
/// a Trainer or Ability with nothing to search for can't be used.
pub struct SearchSpec {
    pub pick: PickSpec,
    pub destination: SearchDestination,
    pub msg: &'static str,
    /// The prompt can be cancelled (`allowCancel`, as the oracle prompt has it).
    pub cancel: bool,
    /// Shuffle the chooser's deck as soon as the prompt opens, before it is answered
    /// (Twinleaf's order for Gimmighoul; the prompt then lists the shuffled deck).
    pub shuffle_first: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SearchDestination {
    /// Onto the chooser's Bench (as played from the zone), at most the open
    /// Bench spaces.
    Bench,
    /// Into the chooser's hand; `reveal` shows the cards to the opponent first.
    Hand { reveal: bool },
    /// Into the chooser's discard pile.
    Discard { reveal: bool },
    /// Onto the bottom of the chooser's deck.
    Deck { reveal: bool },
}
pub struct Bounds {
    pub min: Num,
    pub max: Num,
}
/// Copy the cards of a zone matching a predicate into a register.
pub struct SnapshotSpec {
    pub zone: ZoneRef,
    pub predicate: Pred,
    pub into: u8,
}
pub struct OrderSpec {}
/// Attach Energy chosen from a zone to Pokémon in play (an AttachEnergy prompt).
pub struct AttachSpec {
    pub chooser: Who,
    pub from: ZoneRef,
    /// The cards that can be attached (include the Energy kind).
    pub predicate: Pred,
    pub slots: AttachSlots,
    /// The top card of a Pokémon that can receive the cards.
    pub target: Pred,
    pub scan: TargetScan,
    pub bounds: Bounds,
    pub same_target: bool,
    pub different_targets: bool,
    /// Only these Energy types (empty: any); with `max_per_type` (0: no limit).
    pub valid_types: &'static [u8],
    pub max_per_type: u8,
    pub cancel: bool,
    pub route: AttachRoute,
    /// Shuffle the chooser's deck again when nothing was attached (today's behavior of
    /// Smoochum and Cinderace, whose search shuffles before the answer).
    pub none_shuffles: bool,
}
impl AttachSpec {
    pub const DEFAULT: AttachSpec = AttachSpec {
        chooser: Who::Me,
        from: ZoneRef(Who::Me, Zone::Deck),
        predicate: Pred::BasicEnergy,
        slots: AttachSlots::Bench,
        target: Pred::Any,
        scan: TargetScan::InPlay,
        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
        same_target: false,
        different_targets: false,
        valid_types: &[],
        max_per_type: 0,
        cancel: false,
        route: AttachRoute::Move,
        none_shuffles: false,
    };
}
/// `AttachSpec::max_per_type` value that asks for the prompt's `differentTypes` instead: the chosen
/// cards must differ in type.
pub const DIFFERENT_TYPES: u8 = 255;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachSlots {
    Bench,
    BenchActive,
    // --- S3-4 appends ---
    /// Only the Active Pokémon.
    Active,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetScan {
    /// Every Pokémon in play (Active first) whose top card fails `target` is blocked.
    InPlay,
    /// Benched Pokémon whose current type (Special-condition-proof `CheckPokemonType`)
    /// is not the `Pred::PokemonType` of `target` are blocked.
    BenchEffectiveType,
    // --- S3-4 appends ---
    /// Every Pokémon in play (Active first) whose current type is none of these is blocked.
    EffectiveTypes(&'static [CardType]),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachRoute {
    /// The cards move to the Pokémon (not an attachment from hand).
    Move,
    /// The attachment effect (attached from hand by an Ability).
    Effect,
    // --- S3-4 appends ---
    /// The cards move to the Pokémon, and the deck is shuffled after each one (today's behavior of
    /// Yanmega ex's Buzz Boost).
    MoveShufflePerCard,
}
/// Move an Energy from one Pokémon to another (a MoveEnergy prompt); an attack
/// effect that effect-prevention can stop.
pub struct MoveEnergySpec {
    pub chooser: Who,
    /// Whose Pokémon the Energy moves between.
    pub owner: Who,
}
pub struct DiscardEnergySpec {
    pub target: SlotExpr,
    pub selection: EnergySelection,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergySelection {
    /// Every card providing Energy to the Pokémon.
    AllProvided,
}
/// Put the cards of a register onto a player's Bench, as played from where they are
/// (effect placement: the placed turn is set).
pub struct PlayFromZoneSpec {
    pub cards: u8,
    pub who: Who,
}
/// Team Rocket's Bother-Bot: turn 1 of `whose`'s face-down Prize cards face up (it stays face up), look at a
/// random card of their hand, and you may have them switch those cards.
pub struct PickPrizeSpec {
    pub whose: Who,
}
pub struct PrizeVisibilitySpec {}
/// The player takes 1 of their Prize cards (chosen when more than 1 is left; an attack's choice is made at step D).
pub struct TakePrizeSpec {
    pub who: Who,
}
/// Shuffle a hand into its deck, then draw (the resolving card is not part
/// of the hand).
pub struct HandShuffleDrawSpec {
    pub who: Who,
    pub draw: Num,
}

// ---------------------------------------------------------------------------
// Helpers

fn set_reg(g: &mut Game, f: &mut Frame, r: u8, cards: &[CardId]) {
    let r = r as usize;
    if f.cards[r] == NONE {
        if let ListRef::Temp(i) = g.alloc_temp(cards) {
            f.cards[r] = i;
        }
    } else {
        g.lst_mut(ListRef::Temp(f.cards[r])).set_from(cards);
    }
}

/// Cards in the player's hand, not counting the resolving card (a Trainer is out of the hand while it resolves).
fn hand_len_without(g: &Game, p: usize, me: CardId) -> i32 {
    g.st.players[p].hand.iter().filter(|c| *c != me).count() as i32
}

fn zone_is_unset(f: &Frame, z: ZoneRef) -> bool {
    matches!(z.1, Zone::Scratch(r) if f.cards[r as usize] == NONE)
}

/// The cards of a zone in list order; the resolving card is never part of a hand.
fn zone_cards(g: &Game, me: CardId, f: &Frame, z: ZoneRef) -> Vec<CardId> {
    if zone_is_unset(f, z) {
        return Vec::new();
    }
    let mut v = g.lst(zone_ref(f, z)).to_vec();
    if z.1 == Zone::Hand {
        v.retain(|c| *c != me);
    }
    v
}

fn show_to(g: &mut Game, viewer: usize, n: usize) {
    show_cards_to_player(g, viewer, n);
}

fn to_u8(n: i32) -> u8 {
    n.clamp(0, 255) as u8
}

fn num_uses_reg(n: &Num) -> bool {
    match n {
        Num::RegCount(_) => true,
        Num::Add(a, b) | Num::Sub(a, b) | Num::Mul(a, b) | Num::Min(a, b) | Num::Max(a, b) => num_uses_reg(a) || num_uses_reg(b),
        Num::If(_, a, b) => num_uses_reg(a) || num_uses_reg(b),
        _ => false,
    }
}

fn open_shuffle(g: &mut Game, me: CardId, f: &Frame, p: usize, sub: u8) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, f.cont(me, sub));
}

// ---------------------------------------------------------------------------
// exec

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::Draw(d) => {
            let p = f.who(d.who);
            let n = match &d.amount {
                DrawAmount::Count(n) => num(g, me, f, n),
                DrawAmount::UntilHandSize(n) => num(g, me, f, n) - hand_len_without(g, p, me),
            };
            if n > 0 {
                draw_cards(g, p, n as usize)?;
            }
            Ok(Flow::Next)
        }
        Op::Move(m) => {
            do_move(g, me, f, m)?;
            Ok(Flow::Next)
        }
        Op::Snapshot(s) => {
            let cards: Vec<CardId> = zone_cards(g, me, f, s.zone).into_iter().filter(|c| pred(g, *c, &s.predicate)).collect();
            set_reg(g, f, s.into, &cards);
            Ok(Flow::Next)
        }
        Op::Reveal(r) => {
            let cards = match &r.cards {
                RevealWhat::Zone(z) => zone_cards(g, me, f, *z),
                RevealWhat::Chosen(reg) => reg_list(g, f, *reg).to_vec(),
            };
            if !cards.is_empty() || r.when_empty {
                let id = g.player_id(f.who(r.to));
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
            }
            Ok(Flow::Next)
        }
        Op::Pick(p) => {
            if let Some(c) = f.recorded_choice(g, me) {
                let cards: Vec<CardId> = c.items[..c.len as usize].to_vec();
                set_reg(g, f, p.into, &cards);
                return Ok(Flow::Next);
            }
            if ask_pick(g, me, f, p, i32::MAX, p_msg(p, ""), 1, false) {
                Ok(Flow::Suspend)
            } else {
                set_reg(g, f, p.into, &[]);
                Ok(Flow::Next)
            }
        }
        Op::Search(s) => search_exec(g, me, f, s),
        Op::Shuffle(s) => {
            let p = f.who(s.zone.0);
            open_shuffle(g, me, f, p, 1);
            Ok(Flow::Suspend)
        }
        Op::Attach(a) => {
            if attach_prompt(g, me, f, a)? {
                Ok(Flow::Suspend)
            } else {
                Ok(Flow::Next)
            }
        }
        Op::PlayFromZone(pz) => {
            let p = f.who(pz.who);
            let cards: Vec<CardId> = reg_list(g, f, pz.cards).to_vec();
            let open = empty_bench_slots(g, p);
            for (c, s) in cards.iter().zip(open.iter()) {
                let in_reg = f.cards.iter().filter(|r| **r != NONE).map(|r| ListRef::Temp(*r)).find(|l| g.lst(*l).contains(c));
                if let Some(src) = g.st.locate(*c).or(in_reg) {
                    move_cards(g, src, ListRef::Slot(p as u8, *s), &[*c], me)?;
                    g.st.players[p].slots[*s as usize].pokemon_played_turn = g.st.turn;
                }
            }
            Ok(Flow::Next)
        }
        Op::DiscardEnergy(d) => {
            let Some(slot) = slot_of(g, me, f, d.target) else { return Ok(Flow::Next) };
            let Some((p, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(Flow::Next) };
            match d.selection {
                EnergySelection::AllProvided => {
                    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: slot.p, source: slot, energy_map: SVec::new() })?;
                    let mut cards: SVec<CardId, 64> = SVec::new();
                    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                        for m in energy_map.iter() {
                            cards.push(m.card);
                        }
                    }
                    let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: slot };
                    g.run_fx(Effect::DiscardCards { b, cards })?;
                }
            }
            Ok(Flow::Next)
        }
        Op::MoveEnergy(m) => {
            if let Some(c) = f.recorded_choice(g, me) {
                if c.answer == CHOICE_YES {
                    carry_out_transfers(g, f, &decode_transfers(&c.items[..c.len as usize]))?;
                }
                return Ok(Flow::Next);
            }
            if move_energy_prompt(g, me, f, m, 1) {
                Ok(Flow::Suspend)
            } else {
                Ok(Flow::Next)
            }
        }
        Op::HandShuffleDraw(h) => {
            let p = f.who(h.who);
            let n = num(g, me, f, &h.draw).max(0) as u8;
            shuffle_hand_into_deck_then_draw_ex(g, p, me, NO_CARD, n, Some((me, f.frame_at(1))))?;
            Ok(Flow::Suspend)
        }
        Op::EnergyChoice(e) => ec_exec(g, me, f, e),
        Op::PickPrize(pp) => bother_exec(g, me, f, pp),
        Op::PrizeBonus(pb) => {
            // The Knock Out (a trigger's effect) takes `n` more Prize cards.
            if let Effect::KnockOut { prize_count, .. } = g.e_mut(f.eff) {
                *prize_count += pb.n;
            }
            Ok(Flow::Next)
        }
        Op::TakePrize(t) => {
            if let Some(c) = f.recorded_choice(g, me) {
                if c.answer == CHOICE_YES && c.len > 0 {
                    let p = f.who(t.who);
                    crate::engine::check::take_specific_prizes(g, p, &[c.items[0]], ListRef::Hand(p as u8), false)?;
                }
                return Ok(Flow::Next);
            }
            tp_begin(g, me, f, t, false)
        }
        Op::MoveToSlot(m) => {
            if let Some(dst) = slot_of(g, me, f, m.to) {
                let cards: Vec<CardId> = reg_list(g, f, m.cards).to_vec();
                for c in cards {
                    if let Some(src) = g.st.locate(c) {
                        move_cards(g, src, dst.list(), &[c], me)?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        _ => unimplemented!("spec op not implemented yet (ops/cards.rs)"),
    }
}

fn p_msg(p: &PickSpec, fallback: &'static str) -> &'static str {
    if p.msg.is_empty() {
        if fallback.is_empty() {
            "CHOOSE_CARD"
        } else {
            fallback
        }
    } else {
        p.msg
    }
}

fn do_move(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveSpec) -> R {
    // Where the cards come from and which of them move.
    let (src, mut cards): (ListRef, Vec<CardId>) = match &m.cards {
        CardSel::Stadium => match g.st.stadium_card().and_then(|c| g.st.locate(c).map(|l| (l, c))) {
            Some((l, c)) => {
                let owner = l.owner().unwrap_or(0);
                move_cards(g, l, ListRef::Discard(owner as u8), &[c], me)?;
                return Ok(());
            }
            None => return Ok(()),
        },
        CardSel::Tools(s) => match slot_of(g, me, f, *s) {
            Some(slot) => (slot.list(), g.st.slot(slot.p as usize, slot.s).tools.iter().collect()),
            None => return Ok(()),
        },
        sel => {
            if zone_is_unset(f, m.from) {
                if let Some(r) = m.into {
                    set_reg(g, f, r, &[]);
                }
                return Ok(());
            }
            let zc = zone_cards(g, me, f, m.from);
            let cards = match sel {
                CardSel::All => zc,
                CardSel::Top(n) => zc.into_iter().take(num(g, me, f, n).max(0) as usize).collect(),
                CardSel::Bottom(n) => {
                    let n = (num(g, me, f, n).max(0) as usize).min(zc.len());
                    zc[zc.len() - n..].to_vec()
                }
                CardSel::Random(n) => {
                    let mut pool = zc;
                    let n = (num(g, me, f, n).max(0) as usize).min(pool.len());
                    let mut out = Vec::new();
                    for _ in 0..n {
                        let i = g.rng.index(pool.len());
                        out.push(pool.remove(i));
                    }
                    out
                }
                CardSel::Chosen(r) => reg_list(g, f, *r).to_vec(),
                CardSel::First(pr) => zc.into_iter().filter(|c| pred(g, *c, pr)).take(1).collect(),
                CardSel::Tools(_) | CardSel::Stadium => unreachable!(),
            };
            (zone_ref(f, m.from), cards)
        }
    };
    if m.shuffle_first && !cards.is_empty() {
        let n = cards.len();
        let mut perm = [0u8; 120];
        g.rng.shuffle(n, &mut perm);
        cards = (0..n).map(|i| cards[perm[i] as usize]).collect();
    }
    if let Some(r) = m.into {
        set_reg(g, f, r, &cards);
    }
    if cards.is_empty() {
        return Ok(());
    }
    if let Some(w) = m.reveal {
        show_to(g, f.who(w), cards.len());
    }
    if let Zone::Scratch(r) = m.to.1 {
        if f.cards[r as usize] == NONE {
            set_reg(g, f, r, &[]);
        }
    }
    let dst = zone_ref(f, m.to);
    if matches!(m.cards, CardSel::Tools(_)) {
        for c in cards {
            move_cards(g, src, dst, &[c], me)?;
        }
        return Ok(());
    }
    match m.place {
        Place::End => move_cards(g, src, dst, &cards, me),
        Place::Top => {
            g.run_fx(Effect::MoveCards {
                source: src,
                destination: dst,
                cards: Some(List::from_slice(&cards)),
                count: None,
                to_top: true,
                to_bottom: false,
                skip_cleanup: false,
                source_card: me,
            })?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Pick and Search

/// Open the choice of `pick` (a ChooseCards prompt resumed at `sub`) unless there is
/// nothing to choose: a hidden deck is searched whenever it holds a card; any other
/// zone needs an eligible card. `max_cap` limits the number of cards (open Bench spaces).
fn ask_pick(g: &mut Game, me: CardId, f: &Frame, pick: &PickSpec, max_cap: i32, msg: &'static str, sub: u8, cancel: bool) -> bool {
    let p = f.who(pick.chooser);
    let cards = zone_cards(g, me, f, pick.from);
    if cards.is_empty() {
        return false;
    }
    let eligible = cards.iter().filter(|c| pred(g, **c, &pick.predicate)).count();
    let hidden = pick.from.1 == Zone::Deck;
    let max = num(g, me, f, &pick.bounds.max).min(max_cap).max(0);
    let min = num(g, me, f, &pick.bounds.min).min(max).max(0);
    if (!hidden && (eligible == 0 || max == 0)) || (hidden && max_cap == 0) {
        return false;
    }
    let mut opts = ChooseCardsOpts::new(to_u8(min), to_u8(max), pick.cancel || cancel);
    for (i, c) in cards.iter().enumerate() {
        if !pred(g, *c, &pick.predicate) {
            opts.blocked.push(i as u8);
        }
    }
    opts.different_types = pick.distinct_types;
    for cap in pick.caps {
        let v = Some(to_u8(num(g, me, f, &cap.max)));
        match cap.kind {
            CapKind::Pokemon => opts.max_pokemons = v,
            CapKind::BasicEnergy => opts.max_basic_energies = v,
            CapKind::Energy => opts.max_energies = v,
            CapKind::Trainer => opts.max_trainers = v,
            CapKind::Tool => opts.max_tools = v,
            CapKind::Item => opts.max_items = v,
            CapKind::Stadium => opts.max_stadiums = v,
            CapKind::Supporter => opts.max_supporters = v,
        }
    }
    // A hand's prompt lists it without the resolving card.
    let list = if pick.from.1 == Zone::Hand && g.lst(zone_ref(f, pick.from)).contains(&me) { g.alloc_temp(&cards) } else { zone_ref(f, pick.from) };
    choose_cards(g, p, msg, list, Filter::none(), opts, f.cont(me, sub));
    true
}

fn search_room(g: &Game, f: &Frame, s: &SearchSpec) -> i32 {
    match s.destination {
        SearchDestination::Bench => empty_bench_slots(g, f.who(s.pick.chooser)).len() as i32,
        _ => i32::MAX,
    }
}

fn search_msg(s: &SearchSpec) -> &'static str {
    if !s.pick.msg.is_empty() {
        return s.pick.msg;
    }
    if !s.msg.is_empty() {
        return s.msg;
    }
    match s.destination {
        SearchDestination::Bench => "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
        SearchDestination::Hand { .. } => "CHOOSE_CARD_TO_HAND",
        SearchDestination::Discard { .. } => "CHOOSE_CARD_TO_DISCARD",
        SearchDestination::Deck { .. } => "CHOOSE_CARD_TO_DECK",
    }
}

fn search_exec(g: &mut Game, me: CardId, f: &mut Frame, s: &SearchSpec) -> R<Flow> {
    if let Some(c) = f.recorded_choice(g, me) {
        let cards: Vec<CardId> = c.items[..c.len as usize].to_vec();
        finish_search(g, me, f, s, &cards)?;
        return Ok(Flow::Next);
    }
    if ask_pick(g, me, f, &s.pick, search_room(g, f, s), search_msg(s), 1, s.cancel) {
        if s.shuffle_first {
            open_shuffle(g, me, f, f.who(s.pick.chooser), 9);
        }
        Ok(Flow::Suspend)
    } else {
        set_reg(g, f, s.pick.into, &[]);
        Ok(Flow::Next)
    }
}

fn finish_search(g: &mut Game, me: CardId, f: &mut Frame, s: &SearchSpec, chosen: &[CardId]) -> R {
    let p = f.who(s.pick.chooser);
    set_reg(g, f, s.pick.into, chosen);
    let from = zone_ref(f, s.pick.from);
    let reveal = |g: &mut Game, on: bool| {
        if on && !chosen.is_empty() {
            show_to(g, 1 - p, chosen.len());
        }
    };
    match s.destination {
        SearchDestination::Bench => {
            let open = empty_bench_slots(g, p);
            for (c, slot) in chosen.iter().zip(open.iter()) {
                if matches!(from, ListRef::Deck(_)) {
                    g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, *slot) })?;
                } else {
                    move_cards(g, from, ListRef::Slot(p as u8, *slot), &[*c], me)?;
                    g.st.players[p].slots[*slot as usize].pokemon_played_turn = g.st.turn;
                }
            }
        }
        SearchDestination::Hand { reveal: r } => {
            reveal(g, r);
            move_cards(g, from, ListRef::Hand(p as u8), chosen, me)?;
        }
        SearchDestination::Discard { reveal: r } => {
            reveal(g, r);
            move_cards(g, from, ListRef::Discard(p as u8), chosen, me)?;
        }
        SearchDestination::Deck { reveal: r } => {
            reveal(g, r);
            move_cards(g, from, ListRef::Deck(p as u8), chosen, me)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Attach

fn attach_slots(a: &AttachSpec) -> SVec<u8, 3> {
    let mut slots = SVec::new();
    if a.slots == AttachSlots::Active {
        slots.push(SlotType::Active as u8);
        return slots;
    }
    slots.push(SlotType::Bench as u8);
    if a.slots == AttachSlots::BenchActive {
        slots.push(SlotType::Active as u8);
    }
    slots
}

/// The Pokémon that can't receive the cards (`blockedTo`).
fn blocked_to(g: &mut Game, p: usize, a: &AttachSpec) -> R<SVec<CardTarget, 9>> {
    let mut out: SVec<CardTarget, 9> = SVec::new();
    match a.scan {
        TargetScan::InPlay => {
            for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if !pred(g, c, &a.target) {
                    out.push(t);
                }
            }
        }
        TargetScan::EffectiveTypes(types) => {
            for (slot, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                let target = SlotRef::new(p, slot);
                let now = crate::engine::game_effect::pokemon_types(g, target);
                let (te, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: now })?;
                let ok = matches!(te, Effect::CheckPokemonType { card_types, .. } if types.iter().any(|x| card_types.contains(x)));
                if !ok {
                    out.push(t);
                }
            }
        }
        TargetScan::BenchEffectiveType => {
            let want = match a.target {
                Pred::PokemonType(t) => t,
                _ => unreachable!("BenchEffectiveType needs a PokemonType target"),
            };
            let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
            for (index, s) in bench.into_iter().enumerate() {
                if g.st.slot(p, s).cards.is_empty() {
                    continue;
                }
                let target = SlotRef::new(p, s);
                let types = crate::engine::game_effect::pokemon_types(g, target);
                let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
                let ok = matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&want));
                if !ok {
                    out.push(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, index as u8));
                }
            }
        }
    }
    Ok(out)
}

fn attach_prompt(g: &mut Game, me: CardId, f: &Frame, a: &AttachSpec) -> R<bool> {
    let p = f.who(a.chooser);
    let list = zone_ref(f, a.from);
    let cards = zone_cards(g, me, f, a.from);
    if cards.is_empty() {
        return Ok(false);
    }
    let mut o = AttachOpts::new(cards.len().min(255) as u8);
    o.allow_cancel = a.cancel;
    o.max = to_u8(num(g, me, f, &a.bounds.max));
    o.min = to_u8(num(g, me, f, &a.bounds.min)).min(o.max);
    for (i, c) in cards.iter().enumerate() {
        if !pred(g, *c, &a.predicate) {
            o.blocked.push(i as u8);
        }
    }
    o.blocked_to = blocked_to(g, p, a)?;
    o.same_target = a.same_target;
    o.different_targets = a.different_targets;
    if !a.valid_types.is_empty() {
        let mut vt = SVec::new();
        for t in a.valid_types {
            vt.push(*t);
        }
        o.valid_card_types = Some(vt);
    }
    if a.max_per_type == DIFFERENT_TYPES {
        o.different_types = true;
    } else if a.max_per_type > 0 {
        o.max_per_type = Some(a.max_per_type);
    }
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_CARDS",
        PromptKind::AttachEnergy { cards: list, player_type: PlayerType::BottomPlayer, slots: attach_slots(a), filter: Filter::none(), o },
        f.cont(me, 1),
    );
    Ok(true)
}

fn attach_targets_exist(g: &Game, f: &Frame, a: &AttachSpec) -> bool {
    let p = f.who(a.chooser);
    let scope = if a.slots == AttachSlots::Bench { PlayScope::Bench } else { PlayScope::All };
    in_play(g, p, scope).iter().any(|(_, top, _)| pred(g, *top, &a.target))
}

// ---------------------------------------------------------------------------
// MoveEnergy (an Energy moves between Pokémon of one player)

fn move_energy_prompt(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveEnergySpec, sub: u8) -> bool {
    let owner = f.who(m.owner);
    let pokemon = for_each_pokemon(g, owner, PlayerType::BottomPlayer);
    let has_energy = pokemon.iter().any(|(s, _, _)| g.st.slot(owner, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()));
    if !has_energy || pokemon.len() <= 1 {
        return false;
    }
    let chooser = f.who(m.chooser);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let player_type = if owner == chooser { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let opts = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
    let id = g.player_id(chooser);
    g.prompt(
        id,
        "MOVE_ENERGY_CARDS",
        PromptKind::MoveEnergy { player_type, slots, filter: Filter::super_type(SuperType::Energy), o: opts },
        f.cont(me, sub),
    );
    true
}

/// (from slot, to slot, card) as bytes: slot = player << 4 | slot id.
fn encode_transfers(g: &Game, p: usize, ts: &[(CardTarget, CardTarget, CardId)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (from, to, c) in ts {
        let (Ok(a), Ok(b)) = (get_target(&g.st, p, *from), get_target(&g.st, p, *to)) else { continue };
        out.extend_from_slice(&[a.p << 4 | a.s, b.p << 4 | b.s, *c]);
    }
    out
}

fn decode_transfers(items: &[u8]) -> Vec<(SlotRef, SlotRef, CardId)> {
    items.chunks(3).filter(|c| c.len() == 3).map(|c| (SlotRef { p: c[0] >> 4, s: c[0] & 15 }, SlotRef { p: c[1] >> 4, s: c[1] & 15 }, c[2])).collect()
}

fn carry_out_transfers(g: &mut Game, f: &Frame, ts: &[(SlotRef, SlotRef, CardId)]) -> R {
    let Some((p, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    for (from, to, c) in ts {
        let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: *from };
        g.run_fx(Effect::MoveOpponentEnergy { b, card: *c, destination: *to })?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// resume

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Pick(p) => {
            set_reg(g, f, p.into, first.cards());
            Ok(Flow::Next)
        }
        Op::Search(s) if f.sub == 9 => {
            // The early shuffle's answer: apply it and keep waiting for the choice.
            if let Res::Order(o) = first {
                let p = f.who(s.pick.chooser);
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
                // The open choice lists the reordered deck: its blocked positions follow the cards.
                let deck: Vec<CardId> = g.st.players[p].deck.as_slice().to_vec();
                let blocked: Vec<u8> = (0..deck.len()).filter(|i| !pred(g, deck[*i], &s.pick.predicate)).map(|i| i as u8).collect();
                for pr in g.prompts.as_mut_slice().iter_mut() {
                    if let PromptKind::ChooseCards { cards: ListRef::Deck(d), opts, .. } = &mut pr.kind {
                        if *d as usize == p && pr.result.is_none() {
                            opts.blocked = Blocked::default();
                            for i in &blocked {
                                opts.blocked.push(*i);
                            }
                        }
                    }
                }
            }
            Ok(Flow::Suspend)
        }
        Op::Search(s) => {
            let chosen: Vec<CardId> = first.cards().to_vec();
            finish_search(g, me, f, s, &chosen)?;
            Ok(Flow::Next)
        }
        Op::Shuffle(s) => {
            if let Res::Order(o) = first {
                let p = f.who(s.zone.0);
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(Flow::Next)
        }
        Op::Attach(a) => {
            let p = f.who(a.chooser);
            if f.sub == 2 {
                if let Res::Order(o) = first {
                    crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
                }
                return Ok(Flow::Next);
            }
            let ts: SVec<(CardTarget, CardId), 64> = match first {
                Res::Attach(t) => t,
                _ => SVec::new(),
            };
            if ts.is_empty() {
                if a.none_shuffles && !g.st.players[p].deck.is_empty() {
                    open_shuffle(g, me, f, p, 2);
                    return Ok(Flow::Suspend);
                }
                return Ok(Flow::Next);
            }
            let from = zone_ref(f, a.from);
            for (to, c) in ts.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                // The Pokémon that received the cards is the slot register's.
                f.slot = super::board::encode(target);
                match a.route {
                    AttachRoute::Move => move_cards(g, from, target.list(), &[c], me)?,
                    AttachRoute::Effect => {
                        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
                    }
                    AttachRoute::MoveShufflePerCard => {
                        move_cards(g, from, target.list(), &[c], me)?;
                        shuffle_deck(g, p);
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::MoveEnergy(_) => {
            let ts: Vec<(SlotRef, SlotRef, CardId)> = match first {
                Res::Transfers(ts) => {
                    let p = f.p as usize;
                    ts.iter()
                        .filter_map(|(a, b, c)| Some((get_target(&g.st, p, *a).ok()?, get_target(&g.st, p, *b).ok()?, *c)))
                        .collect()
                }
                _ => Vec::new(),
            };
            carry_out_transfers(g, f, &ts)?;
            Ok(Flow::Next)
        }
        // Resumed after the prefab's draw.
        Op::HandShuffleDraw(_) => Ok(Flow::Next),
        Op::EnergyChoice(e) => ec_resume(g, me, f, e, results, false),
        Op::PickPrize(pp) => bother_resume(g, me, f, pp, first),
        Op::TakePrize(t) => {
            if let Res::Prizes(ix) = first {
                if let Some(i) = ix.as_slice().first() {
                    let p = f.who(t.who);
                    crate::engine::check::take_specific_prizes(g, p, &[*i], ListRef::Hand(p as u8), false)?;
                }
            }
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

// ---------------------------------------------------------------------------
// Step D: attack choices whose options exist before the damage

pub(crate) fn choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::Pick(p) => {
            if ask_pick(g, me, f, p, i32::MAX, p_msg(p, ""), 1, false) {
                return Ok(Flow::Suspend);
            }
            f.record(g, me, CHOICE_NONE);
            set_reg(g, f, p.into, &[]);
            Ok(Flow::Next)
        }
        Op::Search(s) if s.pick.from.1 != Zone::Deck => {
            if ask_pick(g, me, f, &s.pick, search_room(g, f, s), search_msg(s), 1, s.cancel) {
                return Ok(Flow::Suspend);
            }
            f.record(g, me, CHOICE_NONE);
            Ok(Flow::Next)
        }
        Op::MoveEnergy(m) => {
            if move_energy_prompt(g, me, f, m, 1) {
                return Ok(Flow::Suspend);
            }
            f.record(g, me, CHOICE_NONE);
            Ok(Flow::Next)
        }
        Op::EnergyChoice(e) => ec_begin(g, me, f, e, true),
        Op::TakePrize(t) => tp_begin(g, me, f, t, true),
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Pick(p) => {
            f.record_items(g, me, CHOICE_YES, first.cards());
            set_reg(g, f, p.into, first.cards());
            Ok(Flow::Next)
        }
        Op::Search(_) => {
            f.record_items(g, me, CHOICE_YES, first.cards());
            Ok(Flow::Next)
        }
        Op::MoveEnergy(_) => {
            let ts: Vec<(CardTarget, CardTarget, CardId)> = match first {
                Res::Transfers(ts) => ts.iter().copied().collect(),
                _ => Vec::new(),
            };
            let items = encode_transfers(g, f.p as usize, &ts);
            f.record_items(g, me, CHOICE_YES, &items);
            Ok(Flow::Next)
        }
        Op::EnergyChoice(e) => ec_resume(g, me, f, e, results, true),
        Op::TakePrize(_) => {
            match first {
                Res::Prizes(ix) if !ix.is_empty() => f.record_items(g, me, CHOICE_YES, &[ix.as_slice()[0]]),
                _ => f.record(g, me, CHOICE_NONE),
            }
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

// ---------------------------------------------------------------------------
// Preconditions of a Trainer or an Ability

pub(crate) fn implied_ok(g: &Game, me: CardId, f: &Frame, op: &Op) -> bool {
    match op {
        Op::Draw(d) => match &d.amount {
            DrawAmount::Count(n) => num_uses_reg(n) || num(g, me, f, n) > 0,
            DrawAmount::UntilHandSize(n) => num(g, me, f, n) - hand_len_without(g, f.who(d.who), me) > 0,
        },
        Op::Pick(p) => pick_possible(g, me, f, p, i32::MAX),
        Op::Search(s) => pick_possible(g, me, f, &s.pick, search_room(g, f, s)),
        Op::Move(m) => match &m.cards {
            CardSel::Chosen(_) => true,
            CardSel::Tools(_) | CardSel::Stadium => true,
            CardSel::Random(_) | CardSel::All | CardSel::Top(_) | CardSel::Bottom(_) | CardSel::First(_) => !zone_cards(g, me, f, m.from).is_empty() || zone_is_unset(f, m.from),
        },
        Op::Attach(a) => {
            let cards = zone_cards(g, me, f, a.from);
            let eligible = cards.iter().filter(|c| pred(g, **c, &a.predicate)).count() as i32;
            let min = num(g, me, f, &a.bounds.min).max(1);
            let hidden = a.from.1 == Zone::Deck;
            (if hidden { !cards.is_empty() } else { eligible >= min }) && attach_targets_exist(g, f, a)
        }
        Op::PlayFromZone(pz) => !empty_bench_slots(g, f.who(pz.who)).is_empty(),
        _ => true,
    }
}

/// A Trainer or Ability that only chooses cards needs something to choose: a deck
/// that holds a card, or enough eligible cards of a known zone (a Pokémon needs room).
fn pick_possible(g: &Game, me: CardId, f: &Frame, pick: &PickSpec, room: i32) -> bool {
    if pick.soft || zone_is_unset(f, pick.from) {
        return true;
    }
    let cards = zone_cards(g, me, f, pick.from);
    if room <= 0 {
        return false;
    }
    if pick.from.1 == Zone::Deck {
        return !cards.is_empty();
    }
    let eligible = cards.iter().filter(|c| pred(g, **c, &pick.predicate)).count() as i32;
    let min = num(g, me, f, &pick.bounds.min).max(1).min(room);
    eligible >= min
}

// ---------------------------------------------------------------------------
// S3-4 appends: EnergyChoice, MoveToSlot

/// Which Energy cards of a Pokémon can be chosen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergyKind {
    Any,
    /// Energy that provides this type as the Pokémon's Energy provides it now (an Energy that
    /// provides every type counts).
    Provides(CardType),
}

/// Which Pokémon a DiscardEnergy prompt lists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PromptScope {
    Active,
    Bench,
}

/// How the Energy is chosen.
pub enum EnergyHow {
    /// A ChooseCards prompt over the Pokémon's cards (Energy only): between `min` and `max` cards;
    /// with `energies_only` the prompt lists only its Energy cards.
    Cards { min: Num, max: Num, kind: EnergyKind, cancel: bool, energies_only: bool },
    /// A DiscardEnergy prompt over the owner's Active or Benched Pokémon; with `clamp` the numbers
    /// are limited to the Energy cards available.
    Prompt { scope: PromptScope, min: Num, max: Num, kind: EnergyKind, clamp: bool },
    /// A ChooseEnergy prompt: the Energy the Pokémon provides pays `n` Energy of `ty`.
    Cost { n: Num, ty: CardType },
    /// Every Energy card (the cards providing Energy when `provided`); nothing is asked.
    All { provided: bool },
    /// A DiscardEnergy prompt over every Pokémon in play, either side's: `min..=max` Pokémon Tools.
    Tools { min: Num, max: Num },
    /// An AttachEnergy prompt over the Pokémon's cards and the owner's Bench: move `min..=max`
    /// Energy to a Benched Pokémon (an effect of the attack on the owner when `via_effect`).
    ToBench { min: Num, max: Num, same_target: bool, via_effect: bool },
}

/// Where the chosen Energy goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergyDest {
    /// The Pokémon owner's discard pile (the attack's DiscardCards effect in an attack).
    Discard,
    /// The hand of the player the program runs for.
    Hand,
    /// The deck of the player the program runs for.
    Deck,
    /// Onto a Pokémon.
    Slot(SlotExpr),
    /// Nowhere (`ToBench` moves them itself).
    Stay,
}

/// Choose Energy attached to a Pokémon and move it.
pub struct EnergyChoiceSpec {
    /// The Pokémon whose Energy is chosen (a fixed one, or one the chooser picks first). A
    /// `Prompt` takes its owner from the Active Pokémon named here.
    pub from: SlotTarget,
    pub chooser: Who,
    pub how: EnergyHow,
    pub to: EnergyDest,
    /// The register that receives the chosen cards.
    pub into: Option<u8>,
    pub when: Cond,
}
impl EnergyChoiceSpec {
    pub const DEFAULT: EnergyChoiceSpec = EnergyChoiceSpec {
        from: SlotTarget::Slot(SlotExpr::Active(Who::Me)),
        chooser: Who::Me,
        how: EnergyHow::All { provided: true },
        to: EnergyDest::Discard,
        into: None,
        when: Cond::True,
    };
}

/// Put the cards of a register onto a Pokémon (they move there from wherever they are).
pub struct MoveToSlotSpec {
    pub cards: u8,
    pub to: SlotExpr,
}

/// One chosen Energy card: where it is, where it goes, which card.
type Ec = (SlotRef, Option<SlotRef>, CardId);

fn ec_energy_cards(g: &mut Game, src: SlotRef, kind: EnergyKind) -> R<Vec<CardId>> {
    let all: Vec<CardId> = g.st.slot(src.p as usize, src.s).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).collect();
    Ok(match kind {
        EnergyKind::Any => all,
        EnergyKind::Provides(t) => {
            let providing = energy_cards_that_provide_type(g, src.p as usize, src.s, t)?;
            all.into_iter().filter(|c| providing.contains(c)).collect()
        }
    })
}

fn ec_attack_context(g: &Game, f: &Frame) -> bool {
    matches!(f.prog, super::super::run::Prog::Attack(_)) && attack_data(g, f.eff).is_some()
}

/// The owner side a `Prompt` works on: the Active Pokémon named by `from`.
fn ec_owner(f: &Frame, e: &EnergyChoiceSpec) -> usize {
    match &e.from {
        SlotTarget::Slot(SlotExpr::Active(w)) => f.who(*w),
        _ => f.p as usize,
    }
}

fn ec_ptype(g: &Game, f: &Frame, e: &EnergyChoiceSpec, owner: usize) -> PlayerType {
    let _ = g;
    if owner == f.who(e.chooser) {
        PlayerType::BottomPlayer
    } else {
        PlayerType::TopPlayer
    }
}

fn ec_begin(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, record: bool) -> R<Flow> {
    let none = |g: &mut Game, f: &mut Frame| {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        Ok(Flow::Next)
    };
    if !cond_m(g, me, f, &e.when)? {
        return none(g, f);
    }
    if let EnergyHow::Prompt { .. } = &e.how {
        return ec_prompt(g, me, f, e, record);
    }
    if let EnergyHow::Tools { .. } = &e.how {
        return ec_tools(g, me, f, e, record);
    }
    let src = match &e.from {
        SlotTarget::Slot(x) => slot_of(g, me, f, *x),
        SlotTarget::Pick(p) => {
            let cands = super::board::candidates(g, me, f, p)?;
            if cands.is_empty() {
                return none(g, f);
            }
            super::board::ask(g, me, f, p, cands.as_slice(), 1);
            return Ok(Flow::Suspend);
        }
    };
    match src {
        Some(src) if super::board::occupied(g, src) => ec_stage2(g, me, f, e, src, record),
        _ => none(g, f),
    }
}

/// The Pokémon is known: open the prompt for its Energy (or carry out an `All`).
fn ec_stage2(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, src: SlotRef, record: bool) -> R<Flow> {
    let chooser = f.who(e.chooser);
    let sub = 0x80 | super::board::encode(src);
    let none = |g: &mut Game, f: &mut Frame| {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        Ok(Flow::Next)
    };
    let (p, s) = (src.p as usize, src.s);
    match &e.how {
        EnergyHow::Cards { min, max, kind, cancel, energies_only } => {
            let eligible = ec_energy_cards(g, src, *kind)?;
            if eligible.is_empty() {
                return none(g, f);
            }
            let max = num_m(g, me, f, max)?.clamp(0, 255);
            let min = num_m(g, me, f, min)?.clamp(0, max.min(eligible.len() as i32));
            let mut opts = ChooseCardsOpts::new(to_u8(min), to_u8(max), *cancel);
            let list = if *energies_only { ListRef::SlotEnergies(p as u8, s) } else { ListRef::Slot(p as u8, s) };
            for (i, c) in g.lst(list).iter().enumerate() {
                if !eligible.contains(&c) {
                    opts.blocked.push(i as u8);
                }
            }
            choose_cards(g, chooser, "CHOOSE_CARD_TO_DISCARD", list, Filter::super_type(SuperType::Energy), opts, f.cont(me, sub));
            Ok(Flow::Suspend)
        }
        EnergyHow::Cost { n, ty } => {
            let n = num_m(g, me, f, n)?.max(0);
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: src, energy_map: SVec::new() })?;
            let energy = match pe {
                Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
                _ => SVec::new(),
            };
            let mut cost = SVec::new();
            for _ in 0..n {
                cost.push(*ty);
            }
            let id = g.player_id(chooser);
            g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, f.cont(me, sub));
            Ok(Flow::Suspend)
        }
        EnergyHow::All { provided } => {
            if record {
                return Ok(Flow::Next);
            }
            let cards: Vec<CardId> = if *provided {
                let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: src, energy_map: SVec::new() })?;
                match pe {
                    Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.card).collect(),
                    _ => Vec::new(),
                }
            } else {
                ec_energy_cards(g, src, EnergyKind::Any)?
            };
            let ts: Vec<Ec> = cards.into_iter().map(|c| (src, None, c)).collect();
            ec_apply(g, me, f, e, &ts)?;
            Ok(Flow::Next)
        }
        EnergyHow::ToBench { min, max, same_target, .. } => {
            let bench = slots_of(g, me, f, &SlotSel::Bench(if p == f.p as usize { Who::Me } else { Who::Opp }));
            let cards = ec_energy_cards(g, src, EnergyKind::Any)?;
            if bench.is_empty() || cards.is_empty() {
                return none(g, f);
            }
            let total = g.st.slot(p, s).cards.len().min(255) as u8;
            let mut o = AttachOpts::new(total);
            o.allow_cancel = false;
            o.max = to_u8(num_m(g, me, f, max)?).min(cards.len() as u8);
            o.min = to_u8(num_m(g, me, f, min)?).min(o.max);
            o.same_target = *same_target;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let ptype = ec_ptype(g, f, e, p);
            let id = g.player_id(chooser);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, s), player_type: ptype, slots, filter: Filter::super_type(SuperType::Energy), o },
                f.cont(me, sub),
            );
            Ok(Flow::Suspend)
        }
        EnergyHow::Prompt { .. } | EnergyHow::Tools { .. } => unreachable!("handled by ec_prompt / ec_tools"),
    }
}

/// A DiscardEnergy prompt over the Active or Benched Pokémon of the owner.
fn ec_prompt(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, record: bool) -> R<Flow> {
    let EnergyHow::Prompt { scope, min, max, kind, clamp } = &e.how else { return Ok(Flow::Next) };
    let owner = ec_owner(f, e);
    let chooser = f.who(e.chooser);
    let ptype = ec_ptype(g, f, e, owner);
    let in_scope: Vec<(SlotRef, CardTarget)> = {
        let pl = &g.st.players[owner];
        let mut v = Vec::new();
        match scope {
            PromptScope::Active => {
                if !g.st.slot(owner, pl.active).cards.is_empty() {
                    v.push((SlotRef::new(owner, pl.active), CardTarget::new(ptype, SlotType::Active, 0)));
                }
            }
            PromptScope::Bench => {
                for (i, b) in pl.bench.iter().enumerate() {
                    if !g.st.slot(owner, *b).cards.is_empty() {
                        v.push((SlotRef::new(owner, *b), CardTarget::new(ptype, SlotType::Bench, i as u8)));
                    }
                }
            }
        }
        v
    };
    let mut available = 0usize;
    let mut blocked_map: SVec<(CardTarget, Blocked), 18> = SVec::new();
    for (slot, target) in &in_scope {
        let eligible = ec_energy_cards(g, *slot, *kind)?;
        available += eligible.len();
        let mut b = Blocked::default();
        let mut any = false;
        for (i, c) in g.st.slot(owner, slot.s).cards.iter().enumerate() {
            if g.st.cdef(c).is_energy() && !eligible.contains(&c) {
                b.push(i as u8);
                any = true;
            }
        }
        if any {
            blocked_map.push((*target, b));
        }
    }
    if available == 0 {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    let mut max_n = num_m(g, me, f, max)?.max(0);
    let mut min_n = num_m(g, me, f, min)?.max(0);
    if *clamp {
        max_n = max_n.min(available as i32);
    }
    min_n = min_n.min(max_n);
    let mut slots = SVec::new();
    slots.push(match scope {
        PromptScope::Active => SlotType::Active as u8,
        PromptScope::Bench => SlotType::Bench as u8,
    });
    let o = MoveOpts { allow_cancel: false, min: to_u8(min_n), max: Some(to_u8(max_n)), blocked_map, ..Default::default() };
    let id = g.player_id(chooser);
    g.prompt(
        id,
        "CHOOSE_ENERGIES_TO_DISCARD",
        PromptKind::DiscardEnergy { player_type: ptype, slots, filter: Filter::super_type(SuperType::Energy), o },
        f.cont(me, 0xFF),
    );
    Ok(Flow::Suspend)
}

fn ec_exec(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec) -> R<Flow> {
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer != CHOICE_NONE {
            let ts: Vec<Ec> = c.items[..c.len as usize]
                .chunks(3)
                .filter(|t| t.len() == 3)
                .map(|t| (decode_slot(t[0]), if t[1] == NONE { None } else { Some(decode_slot(t[1])) }, t[2]))
                .collect();
            ec_apply(g, me, f, e, &ts)?;
        }
        return Ok(Flow::Next);
    }
    ec_begin(g, me, f, e, false)
}

fn decode_slot(b: u8) -> SlotRef {
    SlotRef { p: b >> 4, s: b & 15 }
}

/// A prompt of an `EnergyChoice` was answered.
fn ec_resume(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, results: &[Res], record: bool) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    if f.sub < 0x80 {
        // The Pokémon was picked.
        let Some(src) = first.slots().first().copied() else {
            if record {
                f.record(g, me, CHOICE_NONE);
            }
            return Ok(Flow::Next);
        };
        if !super::board::occupied(g, src) {
            if record {
                f.record(g, me, CHOICE_NONE);
            }
            return Ok(Flow::Next);
        }
        return ec_stage2(g, me, f, e, src, record);
    }
    let src = if f.sub == 0xFF { None } else { Some(decode_slot(f.sub & 0x7F)) };
    let chooser = f.who(e.chooser);
    let mut ts: Vec<Ec> = Vec::new();
    match (&e.how, first) {
        (EnergyHow::Cards { .. }, Res::Cards(c)) => {
            if let Some(src) = src {
                ts.extend(c.as_slice().iter().map(|x| (src, None, *x)));
            }
        }
        (EnergyHow::Cost { .. }, Res::Energy(c)) => {
            if let Some(src) = src {
                ts.extend(c.as_slice().iter().map(|x| (src, None, *x)));
            }
        }
        (EnergyHow::Prompt { .. } | EnergyHow::Tools { .. }, Res::CardsFrom(t)) => {
            for (from, c) in t.iter().copied() {
                if let Ok(slot) = get_target(&g.st, chooser, from) {
                    ts.push((slot, None, c));
                }
            }
        }
        (EnergyHow::ToBench { .. }, Res::Attach(t)) => {
            if let Some(src) = src {
                for (to, c) in t.iter().copied() {
                    if let Ok(dst) = get_target(&g.st, chooser, to) {
                        ts.push((src, Some(dst), c));
                    }
                }
            }
        }
        _ => {}
    }
    if record {
        let mut items: Vec<u8> = Vec::new();
        for (a, b, c) in ts.iter().take(5) {
            items.extend_from_slice(&[super::board::encode(*a), b.map(super::board::encode).unwrap_or(NONE), *c]);
        }
        let cards: Vec<CardId> = ts.iter().map(|t| t.2).collect();
        f.record_items(g, me, CHOICE_YES, &items);
        if let Some(r) = e.into {
            set_reg(g, f, r, &cards);
        }
    } else {
        ec_apply(g, me, f, e, &ts)?;
    }
    Ok(Flow::Next)
}

/// Carry out the chosen moves.
fn ec_apply(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, ts: &[Ec]) -> R {
    let tools = matches!(e.how, EnergyHow::Tools { .. });
    let ts: Vec<Ec> = ts.iter().copied().filter(|(src, _, c)| tools || g.lst(src.list()).contains(c)).collect();
    if let Some(r) = e.into {
        let cards: Vec<CardId> = ts.iter().map(|t| t.2).collect();
        set_reg(g, f, r, &cards);
    }
    // One move per source Pokémon, in the order they were chosen.
    let mut sources: Vec<SlotRef> = Vec::new();
    for (src, _, _) in &ts {
        if !sources.contains(src) {
            sources.push(*src);
        }
    }
    let attack = ec_attack_context(g, f);
    let me_p = f.p as usize;
    for src in sources {
        let cards: Vec<CardId> = ts.iter().filter(|t| t.0 == src).map(|t| t.2).collect();
        match (&e.how, e.to) {
            (EnergyHow::ToBench { via_effect, .. }, _) => {
                for (a, dst, c) in ts.iter().filter(|t| t.0 == src) {
                    let Some(dst) = dst else { continue };
                    if *via_effect && attack {
                        if let Some((p, opp, attack, source)) = attack_data(g, f.eff) {
                            let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: *a };
                            g.run_fx(Effect::MoveOpponentEnergy { b, card: *c, destination: *dst })?;
                        }
                    } else {
                        move_cards(g, a.list(), dst.list(), &[*c], me)?;
                    }
                }
            }
            (_, EnergyDest::Discard) => {
                if attack {
                    if let Some((p, opp, attack, source)) = attack_data(g, f.eff) {
                        let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: src };
                        let mut cs: SVec<CardId, 64> = SVec::new();
                        for c in &cards {
                            cs.push(*c);
                        }
                        g.run_fx(Effect::DiscardCards { b, cards: cs })?;
                    }
                } else {
                    move_cards(g, src.list(), ListRef::Discard(src.p), &cards, me)?;
                }
            }
            (_, EnergyDest::Hand) => move_cards(g, src.list(), ListRef::Hand(me_p as u8), &cards, me)?,
            (_, EnergyDest::Deck) => move_cards(g, src.list(), ListRef::Deck(me_p as u8), &cards, me)?,
            (_, EnergyDest::Slot(x)) => {
                if let Some(dst) = slot_of(g, me, f, x) {
                    move_cards(g, src.list(), dst.list(), &cards, me)?;
                }
            }
            (_, EnergyDest::Stay) => {}
        }
    }
    Ok(())
}

/// `TAKE_X_PRIZES(1)`: the only Prize card left is taken at once, otherwise the player chooses.
fn tp_begin(g: &mut Game, me: CardId, f: &mut Frame, t: &TakePrizeSpec, record: bool) -> R<Flow> {
    let p = f.who(t.who);
    let left = g.st.players[p].prize_left();
    if left == 0 {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    if left <= 1 {
        let pl = &g.st.players[p];
        let first = (0..pl.prize_count).find(|i| !pl.prizes[*i as usize].is_empty());
        if let Some(i) = first {
            if record {
                f.record_items(g, me, CHOICE_YES, &[i]);
            } else {
                crate::engine::check::take_specific_prizes(g, p, &[i], ListRef::Hand(p as u8), false)?;
            }
        }
        return Ok(Flow::Next);
    }
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize { count: 1, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: false, is_secret: false, destination: None, face_down_only: false },
        f.cont(me, 1),
    );
    Ok(Flow::Suspend)
}

/// A DiscardEnergy prompt over the Pokémon Tools attached to any Pokémon in play.
fn ec_tools(g: &mut Game, me: CardId, f: &mut Frame, e: &EnergyChoiceSpec, record: bool) -> R<Flow> {
    let EnergyHow::Tools { min, max } = &e.how else { return Ok(Flow::Next) };
    let chooser = f.who(e.chooser);
    let mut total = 0usize;
    for q in 0..2usize {
        for (s, _, _) in for_each_pokemon(g, q, PlayerType::BottomPlayer).iter() {
            total += g.st.slot(q, *s).tools.len();
        }
    }
    if total == 0 {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        return Ok(Flow::Next);
    }
    let max_n = num_m(g, me, f, max)?.clamp(0, 255);
    let min_n = num_m(g, me, f, min)?.clamp(0, max_n);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
    let o = MoveOpts { allow_cancel: false, min: to_u8(min_n), max: Some(to_u8(max_n)), ..Default::default() };
    let id = g.player_id(chooser);
    g.prompt(id, "CHOOSE_CARD_TO_DISCARD", PromptKind::DiscardEnergy { player_type: PlayerType::Any, slots, filter, o }, f.cont(me, 0xFF));
    Ok(Flow::Suspend)
}

// Team Rocket's Bother-Bot (PickPrize)

/// Key of the side copy of the chosen Prize index and hand card across the prompts.
const BOTHER_STASH: u64 = u64::MAX - 1;

fn bother_show(g: &mut Game, me: CardId, f: &Frame, sub: u8) {
    let id = g.player_id(f.p as usize);
    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, f.cont(me, sub));
}

fn bother_exec(g: &mut Game, me: CardId, f: &mut Frame, pp: &PickPrizeSpec) -> R<Flow> {
    let o = f.who(pp.whose);
    let pl = &g.st.players[o];
    let mut blocked: SVec<u8, 6> = SVec::new();
    let mut n = 0u8;
    for i in 0..pl.prize_count {
        if pl.prizes[i as usize].is_empty() {
            continue;
        }
        if pl.prize_face_up[i as usize] {
            blocked.push(n);
        }
        n += 1;
    }
    if blocked.len() >= n as usize {
        // No face-down Prize card to turn face up: you only look at a random card of their hand.
        let n = g.st.players[o].hand.len();
        if n == 0 {
            return Ok(Flow::Next);
        }
        let _ = g.rng.index(n);
        bother_show(g, me, f, 4);
        return Ok(Flow::Suspend);
    }
    let id = g.player_id(f.p as usize);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize { count: 1, blocked, use_opponent_prizes: true, allow_cancel: false, is_secret: false, destination: None, face_down_only: true },
        f.cont(me, 1),
    );
    Ok(Flow::Suspend)
}

fn bother_resume(g: &mut Game, me: CardId, f: &mut Frame, pp: &PickPrizeSpec, first: Res) -> R<Flow> {
    let o = f.who(pp.whose);
    match f.sub {
        1 => {
            let idx = match first {
                Res::Prizes(ix) if ix.len() >= 1 => ix.as_slice()[0],
                _ => crate::bail!("INVALID_PROMPT_RESULT"),
            };
            if g.st.players[o].prize_face_up[idx as usize] {
                crate::bail!("INVALID_PROMPT_RESULT");
            }
            // That Prize card remains face up for the rest of the game.
            g.st.players[o].prize_face_up[idx as usize] = true;
            g.st.players[o].prize_public[idx as usize] = true;
            let n = g.st.players[o].hand.len();
            if n == 0 {
                bother_show(g, me, f, 4);
                return Ok(Flow::Suspend);
            }
            let hand_card = g.st.players[o].hand.as_slice()[g.rng.index(n)];
            g.spec_choices.retain(|x| !(x.card == me && x.key == BOTHER_STASH));
            let mut ch = SpecChoice { card: me, key: BOTHER_STASH, answer: 0, items: [0; 16], len: 2 };
            ch.items[0] = idx;
            ch.items[1] = hand_card;
            g.spec_choices.push(ch);
            bother_show(g, me, f, 2);
            Ok(Flow::Suspend)
        }
        2 => {
            confirmation_prompt(g, f.p as usize, "WANT_TO_USE_ABILITY", f.cont(me, 3));
            Ok(Flow::Suspend)
        }
        3 => {
            let Some(i) = g.spec_choices.iter().position(|x| x.card == me && x.key == BOTHER_STASH) else { return Ok(Flow::Next) };
            let st = g.spec_choices.remove_at(i);
            if first.as_bool() {
                let (idx, hand_card) = (st.items[0], st.items[1] as CardId);
                let prize = ListRef::Prize(o as u8, idx);
                let prize_cards: Vec<CardId> = g.lst(prize).to_vec();
                move_cards(g, prize, ListRef::Hand(o as u8), &prize_cards, me)?;
                move_cards(g, ListRef::Hand(o as u8), prize, &[hand_card], me)?;
                g.st.players[o].prize_face_up[idx as usize] = true;
                g.st.players[o].prize_public[idx as usize] = true;
            }
            Ok(Flow::Next)
        }
        _ => Ok(Flow::Next),
    }
}

/// The Knock Out the trigger reacts to gives `n` more Prize cards.
pub struct PrizeBonusSpec {
    pub n: i32,
}
