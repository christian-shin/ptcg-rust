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
    // --- S3 agent 3 appends ---
    /// The Stadium in play, from the zone it is in to its owner's discard pile (`from` and `to` are ignored).
    Stadium,
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
    /// Like `UntilHandSize`, not counting the resolving card (a Supporter still in hand).
    UntilHandSizeOthers(Num),
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachSlots {
    Bench,
    BenchActive,
    // --- S3 agent 3 appends ---
    /// The Active Pokémon, then the Bench (the prompt lists the slot types in this order).
    ActiveBench,
    /// The Active Pokémon only.
    ActiveOnly,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetScan {
    /// Every Pokémon in play (Active first) whose top card fails `target` is blocked.
    InPlay,
    /// Benched Pokémon whose current type (Special-condition-proof `CheckPokemonType`)
    /// is not the `Pred::PokemonType` of `target` are blocked.
    BenchEffectiveType,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachRoute {
    /// The cards move to the Pokémon (not an attachment from hand).
    Move,
    /// The attachment effect (attached from hand by an Ability).
    Effect,
}
/// Move an Energy from one Pokémon to another (a MoveEnergy prompt); an attack
/// effect that effect-prevention can stop.
pub struct MoveEnergySpec {
    pub chooser: Who,
    /// Whose Pokémon the Energy moves between.
    pub owner: Who,
    pub mode: MoveEnergyMode,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveEnergyMode {
    /// One Energy moves between the owner's Pokémon, as an effect of the attack.
    Effect,
    // --- S3 agent 3 appends ---
    /// Up to `max` Energy cards of the Benched Pokémon move to the Active Pokémon (N's Plot: at
    /// least 1, or 0 when the Trainer is used as the effect of an attack).
    BenchToActive { max: u8 },
    /// One Basic Energy card named `name` moves from one of the owner's Pokémon to another
    /// (a plain move, not an attack effect).
    BasicNamed { name: &'static str },
}
pub struct DiscardEnergySpec {
    pub target: SlotExpr,
    pub selection: EnergySelection,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergySelection {
    /// Every card providing Energy to the Pokémon.
    AllProvided,
    // --- S3 agent 3 appends ---
    /// "Discard N Energy from this Pokémon": the player pays `count` Energy of type `ty` from the
    /// Energy the Pokémon provides (a ChooseEnergy prompt, no cancel; never more cards than `count`).
    /// With a type other than [C], nothing happens when no Energy provides it.
    Choose { count: u8, ty: CardType },
    /// "Discard any amount / up to N Energy ...; N damage for each card discarded": a DiscardEnergy
    /// prompt (not cancellable, 0 allowed). The attack's damage becomes `damage_per` times the
    /// number of cards discarded (0 when nothing is discarded).
    Among(AmongSpec),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AmongSpec {
    /// Energy of all the player's Pokémon (otherwise the Active Pokémon's only).
    pub all_pokemon: bool,
    pub which: AmongWhich,
    /// At most this many cards (otherwise as many as there are to choose).
    pub max: Option<u8>,
    pub damage_per: i32,
    /// The Energy leaves after the damage, one move per card (otherwise one DiscardCards effect
    /// per Pokémon, at once).
    pub after_damage: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AmongWhich {
    Any,
    Basic,
    /// Energy that provides the type as it is provided now (Legacy Energy counts).
    Provides(CardType),
}
/// Put the cards of a register onto a player's Bench, as played from where they are
/// (effect placement: the placed turn is set).
pub struct PlayFromZoneSpec {
    pub cards: u8,
    pub who: Who,
}
pub struct PickPrizeSpec {}
pub struct PrizeVisibilitySpec {}
pub struct TakePrizeSpec {}
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

fn zone_is_unset(f: &Frame, z: ZoneRef) -> bool {
    matches!(z.1, Zone::Scratch(r) if f.cards[r as usize] == NONE) || (z.1 == Zone::PickedSlot && f.slot == NONE)
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
                DrawAmount::Count(n) => num_m(g, me, f, n)?,
                DrawAmount::UntilHandSize(n) => num_m(g, me, f, n)? - g.st.players[p].hand.len() as i32,
                DrawAmount::UntilHandSizeOthers(n) => num(g, me, f, n) - g.st.players[p].hand.iter().filter(|c| *c != me).count() as i32,
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
                EnergySelection::Choose { count, ty } => return discard_choose_exec(g, me, f, slot, count, ty),
                EnergySelection::Among(a) => return among_exec(g, me, f, slot, &a),
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
            if m.mode != MoveEnergyMode::Effect {
                return Ok(if move_energy_mode_prompt(g, me, f, m) { Flow::Suspend } else { Flow::Next });
            }
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
    if matches!(m.cards, CardSel::Stadium) {
        // `MOVE_CARDS(findCardList(stadium), findOwner(list).discard, { sourceCard })`.
        let Some(stadium) = g.st.stadium_card() else { return Ok(()) };
        let Some(src) = g.st.locate(stadium) else { return Ok(()) };
        let Some(owner) = src.owner() else { return Ok(()) };
        g.run_fx(Effect::MoveCards {
            source: src,
            destination: ListRef::Discard(owner as u8),
            cards: None,
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
        return Ok(());
    }
    // Where the cards come from and which of them move.
    let (src, mut cards): (ListRef, Vec<CardId>) = match &m.cards {
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
    if a.slots == AttachSlots::ActiveOnly {
        slots.push(SlotType::Active as u8);
        return slots;
    }
    if a.slots == AttachSlots::ActiveBench {
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
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
    if a.max_per_type > 0 {
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
                match a.route {
                    AttachRoute::Move => move_cards(g, from, target.list(), &[c], me)?,
                    AttachRoute::Effect => {
                        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::MoveEnergy(m) if m.mode != MoveEnergyMode::Effect => {
            let Res::Transfers(ts) = first else { return Ok(Flow::Next) };
            let p = f.p as usize;
            for (from, to, card) in ts.iter() {
                let src = get_target(&g.st, p, *from)?;
                let dst = match m.mode {
                    // Today's behavior: every transfer goes to the Active Pokémon whatever the
                    // chosen destination.
                    MoveEnergyMode::BenchToActive { .. } => SlotRef::new(p, g.st.players[p].active),
                    _ => get_target(&g.st, p, *to)?,
                };
                move_cards(g, src.list(), dst.list(), &[*card], me)?;
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
        Op::DiscardEnergy(d) => {
            if let EnergySelection::Among(a) = d.selection {
                among_resume(g, me, f, &a, first)?;
                return Ok(Flow::Next);
            }
            if let Some(slot) = slot_of(g, me, f, d.target) {
                if let Res::Energy(c) = first {
                    discard_chosen(g, f, slot, c.as_slice())?;
                }
            }
            Ok(Flow::Next)
        }
        // Resumed after the prefab's draw.
        Op::HandShuffleDraw(_) => Ok(Flow::Next),
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
        Op::DiscardEnergy(d) => {
            let EnergySelection::Choose { count, ty } = d.selection else { return Ok(Flow::Next) };
            let Some(slot) = slot_of(g, me, f, d.target) else { return Ok(Flow::Next) };
            if discard_choose_prompt(g, me, f, slot, count, ty)? {
                return Ok(Flow::Suspend);
            }
            f.record(g, me, CHOICE_NONE);
            Ok(Flow::Next)
        }
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
        Op::DiscardEnergy(_) => {
            match first {
                Res::Energy(c) => f.record_items(g, me, CHOICE_YES, c.as_slice()),
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
            DrawAmount::Count(n) => num_uses_reg(n) || num_is_checked(n) || num(g, me, f, n) > 0,
            // A checked count can't be read here: assumed possible.
            DrawAmount::UntilHandSize(n) if num_is_checked(n) => true,
            DrawAmount::UntilHandSize(n) => num(g, me, f, n) - g.st.players[f.who(d.who)].hand.len() as i32 > 0,
            // The card's own steps make room to draw (Naveen discards first): its `needs` decide.
            DrawAmount::UntilHandSizeOthers(_) => true,
        },
        Op::Pick(p) => pick_possible(g, me, f, p, i32::MAX),
        Op::Search(s) => pick_possible(g, me, f, &s.pick, search_room(g, f, s)),
        Op::Move(m) => match &m.cards {
            CardSel::Chosen(_) => true,
            CardSel::Tools(_) => true,
            CardSel::Stadium => g.st.stadium_card().is_some(),
            CardSel::Random(_) | CardSel::All | CardSel::Top(_) | CardSel::Bottom(_) => !zone_cards(g, me, f, m.from).is_empty() || zone_is_unset(f, m.from),
        },
        Op::Attach(a) => {
            let cards = zone_cards(g, me, f, a.from);
            let eligible = cards.iter().filter(|c| pred(g, **c, &a.predicate)).count() as i32;
            let min = num(g, me, f, &a.bounds.min).max(1);
            let hidden = a.from.1 == Zone::Deck;
            // A register filled by an earlier step can't be judged yet.
            (zone_is_unset(f, a.from) || if hidden { !cards.is_empty() } else { eligible >= min }) && attach_targets_exist(g, f, a)
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
// DiscardEnergy::Choose (S3 agent 3)

/// Open the ChooseEnergy prompt for `count` Energy of `ty` among the Energy the Pokémon provides.
/// Returns false (no prompt) when there is nothing to pay with.
fn discard_choose_prompt(g: &mut Game, me: CardId, f: &Frame, slot: SlotRef, count: u8, ty: CardType) -> R<bool> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: slot.p, source: slot, energy_map: SVec::new() })?;
    let energy = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    if ty != ct::COLORLESS && !energy.iter().any(|m| m.provides.iter().any(|t| *t == ty || *t == ct::ANY)) {
        return Ok(false);
    }
    let mut cost = SVec::new();
    for _ in 0..count {
        cost.push(ty);
    }
    let id = g.player_id(f.p as usize);
    g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, f.cont(me, 1));
    Ok(true)
}

fn discard_choose_exec(g: &mut Game, me: CardId, f: &mut Frame, slot: SlotRef, count: u8, ty: CardType) -> R<Flow> {
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer == CHOICE_YES {
            discard_chosen(g, f, slot, &c.items[..c.len as usize])?;
        }
        return Ok(Flow::Next);
    }
    if discard_choose_prompt(g, me, f, slot, count, ty)? {
        Ok(Flow::Suspend)
    } else {
        Ok(Flow::Next)
    }
}

/// A DiscardCards effect of the attack on the Pokémon.
fn discard_chosen(g: &mut Game, f: &Frame, slot: SlotRef, cards: &[CardId]) -> R {
    let Some((p, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    let mut cs: SVec<CardId, 64> = SVec::new();
    for c in cards {
        cs.push(*c);
    }
    let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: slot };
    g.run_fx(Effect::DiscardCards { b, cards: cs })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// MoveEnergy modes other than the attack effect (S3 agent 3)

/// Open the MoveEnergy prompt of a `BenchToActive` or `BasicNamed` move; false when there is nothing to move.
fn move_energy_mode_prompt(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveEnergySpec) -> bool {
    let owner = f.who(m.owner);
    let chooser = f.who(m.chooser);
    let pokemon = for_each_pokemon(g, owner, PlayerType::BottomPlayer);
    let player_type = if owner == chooser { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let mut slots = SVec::new();
    let mut o = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
    let filter = match m.mode {
        MoveEnergyMode::BasicNamed { name } => {
            let named = |g: &Game, c: CardId| {
                let d = g.st.cdef(c);
                d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == name
            };
            let has = pokemon.iter().any(|(s, _, _)| g.st.slot(owner, *s).cards.iter().any(|c| named(g, c)));
            if !has || pokemon.len() < 2 {
                return false;
            }
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some(name), ..Filter::none() }
        }
        MoveEnergyMode::BenchToActive { max } => {
            let has = pokemon.iter().any(|(s, _, t)| t.slot == SlotType::Bench && g.st.slot(owner, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()));
            if !has || pokemon.len() <= 1 {
                return false;
            }
            o.min = if f.via_attack { 0 } else { 1 };
            o.max = Some(max);
            for (s, _, t) in pokemon.iter() {
                if t.slot == SlotType::Active {
                    let mut b = Blocked::default();
                    for i in 0..g.st.slot(owner, *s).cards.len() {
                        b.push(i as u8);
                    }
                    o.blocked_map.push((*t, b));
                } else {
                    o.blocked_to.push(*t);
                }
            }
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            Filter::super_type(SuperType::Energy)
        }
        MoveEnergyMode::Effect => return false,
    };
    let id = g.player_id(chooser);
    g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type, slots, filter, o }, f.cont(me, 1));
    true
}

// ---------------------------------------------------------------------------
// DiscardEnergy::Among (S3 agent 3)

fn among_exec(g: &mut Game, me: CardId, f: &mut Frame, slot: SlotRef, a: &AmongSpec) -> R<Flow> {
    let p = f.p as usize;
    if a.damage_per != 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
            *damage = 0;
        }
    }
    let slots_in_scope: Vec<SlotId> = if a.all_pokemon { for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().map(|(s, _, _)| *s).collect() } else { vec![slot.s] };
    let mut o = MoveOpts { allow_cancel: false, min: 0, max: None, ..Default::default() };
    let mut total = 0usize;
    for s in &slots_in_scope {
        match a.which {
            AmongWhich::Any => total += g.st.slot(p, *s).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).count(),
            AmongWhich::Basic => {
                total += g.st.slot(p, *s).cards.iter().filter(|c| {
                    let d = g.st.cdef(*c);
                    d.is_energy() && d.energy_type == EnergyType::Basic as u8
                }).count()
            }
            AmongWhich::Provides(ty) => total += energy_cards_that_provide_type(g, p, *s, ty)?.len(),
        }
    }
    if let AmongWhich::Provides(ty) = a.which {
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if slots_in_scope.contains(&s) {
                if let Some(b) = blocked_non_type_energy(g, p, s, ty)? {
                    o.blocked_map.push((t, b));
                }
            }
        }
    }
    if total == 0 && a.max.is_none() {
        return Ok(Flow::Next);
    }
    o.max = Some(a.max.unwrap_or(total.min(255) as u8));
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    if a.all_pokemon {
        slots.push(SlotType::Bench as u8);
    }
    let filter = if a.which == AmongWhich::Basic {
        Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() }
    } else {
        Filter::super_type(SuperType::Energy)
    };
    let id = g.player_id(p);
    g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, f.cont(me, 1));
    Ok(Flow::Suspend)
}

fn among_resume(g: &mut Game, me: CardId, f: &Frame, a: &AmongSpec, first: Res) -> R {
    let p = f.p as usize;
    let Res::CardsFrom(transfers) = first else { return Ok(()) };
    if transfers.is_empty() {
        return Ok(());
    }
    if a.damage_per != 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
            *damage = transfers.len() as i32 * a.damage_per;
        }
    }
    if a.after_damage {
        for (from, c) in transfers.iter().copied() {
            let s = get_target(&g.st, p, from)?;
            move_cards_after_damage(g, f.eff, s.list(), ListRef::Discard(p as u8), &[c], me)?;
        }
        return Ok(());
    }
    let mut groups: Vec<(SlotRef, SVec<CardId, 64>)> = Vec::new();
    for (from, c) in transfers.iter() {
        let s = get_target(&g.st, p, *from)?;
        match groups.iter_mut().find(|(t, _)| *t == s) {
            Some((_, v)) => v.push(*c),
            None => {
                let mut v = SVec::new();
                v.push(*c);
                groups.push((s, v));
            }
        }
    }
    let Some((_, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    for (target, cards) in groups {
        let b = AtkBase { attack_effect: f.eff, player: p as u8, opponent: opp, attack, source, target };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}
