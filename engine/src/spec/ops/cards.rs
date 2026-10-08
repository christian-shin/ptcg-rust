//! Moving cards between zones, Energy, Prizes (vocabulary v1 "Operations",
//! cards and zones).
//!
//! Chosen cards travel in card registers (`Frame::cards`, scratch lists):
//! `Pick` and `Search` fill register `into`, `Move` can fill one, and later
//! steps read them (`CardSel::Chosen`, `Cond::Chosen`, `Num::RegCount`).
//! Attack choices whose options exist before the damage are made at step D
//! (`choice` / `resume_choice`) and recorded as card ids.

use super::super::run::{Flow, Frame, CHOICE_NONE, CHOICE_YES, NONE};
use super::board::encode;
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
    /// This card itself, from wherever it is (`from` is ignored).
    This,
    /// The first card of the zone matching the predicate (the resolving card never counts); nothing is asked.
    First(Pred),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    /// Appended to the destination (the bottom of a deck).
    End,
    Top,
    /// Attached to this Pokémon (`to` is ignored).
    AttachTo(SlotExpr),
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
    /// Only cards named like one of the Pokémon in play of this player (Love Ball).
    pub same_name_as: Option<Who>,
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
        same_name_as: None,
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
    /// Basic Pokémon.
    Basic,
    /// Evolution Pokémon (Stage 1 or Stage 2).
    Evolution,
    Stage1,
    Stage2,
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
    /// Wait for the answer of the shuffle prompt (otherwise the next step runs while it is still open).
    pub wait: bool,
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
    /// Attached to one of the chooser's Pokémon, picked after the cards are (a ChoosePokemon prompt;
    /// nothing is asked when no card was chosen).
    AttachToPicked,
    /// Attached to this Pokémon as a Pokémon Tool (a Tool found by Impromptu Carrier): the card moves to
    /// the slot's Tools directly.
    AttachToolToThis,
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
/// Put the cards of a register in the order `who` picks (an OrderCards prompt).
pub struct OrderSpec {
    pub who: Who,
    pub zone: ZoneRef,
    pub msg: &'static str,
}
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
    /// The chosen cards must differ in type (the prompt's `differentTypes`).
    pub different_types: bool,
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
        different_types: false,
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
    /// `Move`; the chooser's Active Pokémon is now Poisoned (directly) when a card went to it
    /// (Janine's Secret Art).
    MovePoisonActive,
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
    pub mode: MoveEnergyMode,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveEnergyMode {
    /// One Energy moves between the owner's Pokémon, as an effect of the attack.
    Effect,
    // --- S3 agent 3 appends ---
    /// Up to `max` Energy cards of the Benched Pokémon move to the Active Pokémon (N's Plot: at
    /// least 1, or 0 when the Trainer is used as the effect of an attack).
    /// `ability`: an Ability's version (Iron Leaves ex) of the same rule as a Supporter's (N's Plan), as the
    /// game plays it: the Active Pokémon is blocked as a source in the prompt's `blockedFrom` list (otherwise
    /// as a `blockedMap` entry holding all its cards), and the cards move one source at a time.
    BenchToActive { max: Option<u8>, required: bool, ability: bool },
}
/// Discard, or otherwise move, the Energy (or other cards) of a Pokémon: `selection` says which cards and
/// who chooses them; `to` where they go (the ones of an attack choose at step D, move after the damage).
pub struct DiscardEnergySpec {
    /// The Pokémon whose cards are chosen: a fixed one, or one the chooser picks first (`Cards`, `Cost`, `All`,
    /// `ToBench`). `Scoped` takes the owner side from the Active Pokémon named here.
    pub target: SlotTarget,
    pub selection: EnergySelection,
    /// Who makes the choice (`Cards`, `Scoped`, `Cost`, `Tools`, `ToBench`).
    pub chooser: Who,
    /// Where the chosen cards go (the same selections).
    pub to: EnergyDest,
    /// The register that receives the chosen cards (the same selections).
    pub into: Option<u8>,
    pub when: Cond,
}
impl DiscardEnergySpec {
    pub const DEFAULT: DiscardEnergySpec = DiscardEnergySpec {
        target: SlotTarget::Slot(SlotExpr::Active(Who::Me)),
        selection: EnergySelection::AllProvided,
        chooser: Who::Me,
        to: EnergyDest::Discard,
        into: None,
        when: Cond::True,
    };
}
pub enum EnergySelection {
    /// Every card providing Energy to the Pokémon.
    AllProvided,
    /// Every card providing Energy of this type (or every type) to the Pokémon.
    Provides(CardType),
    /// A DiscardEnergy prompt over the Pokémon's Energy: `min..=max` cards, the cards go to register `into`
    /// when given. With a type, Energy that does not provide `ty` is blocked and the bounds are as written
    /// (an attack's text before the damage); without, the bounds are limited by what is attached and
    /// nothing is asked without Energy.
    Prompt { ty: Option<CardType>, min: u8, max: u8, into: Option<u8> },
    /// The Energy cards of register `r` (chosen earlier), as a DiscardCardsEffect of the attack.
    Register(u8),
    /// Every Special Energy card attached to the Pokémon (an effect of the attack that Mist
    /// Energy and the like can prevent).
    Special,
    /// "Discard N Energy from this Pokémon": the player pays `count` Energy of type `ty` from the
    /// Energy the Pokémon provides (a ChooseEnergy prompt, no cancel; never more cards than `count`).
    /// With a type other than [C], nothing happens when no Energy provides it.
    Choose { count: u8, ty: CardType },
    /// "Discard any amount / up to N Energy ...; N damage for each card discarded": a DiscardEnergy
    /// prompt (not cancellable, 0 allowed). The attack's damage becomes `damage_per` times the
    /// number of cards discarded (0 when nothing is discarded).
    Among(AmongSpec),
    /// "Shuffle all Energy from this Pokémon into your deck": every Energy card of the Pokémon goes
    /// to the deck, which is then shuffled (both after the damage of an attack).
    AllIntoDeck,
    /// Like `Choose`, but the cards go to the hand of the Pokémon's owner (a CardsToHand effect
    /// of the attack); with `up_to` the payment is at most `count` (fewer when the Pokémon
    /// provides fewer Energy).
    ChooseToHand { count: u8, ty: CardType, up_to: bool },
    /// "Discard up to `max` Pokémon Tools from your opponent's Pokémon" (`target` is ignored): a
    /// prompt over the Tools in play; each chosen Tool is checked against an effect that prevents
    /// the attack's effects on its Pokémon (Mist Energy) before it is discarded.
    OppTools { max: u8 },
    /// Exactly one card of the Pokémon matching the predicate, picked in a ChooseCards prompt
    /// (nothing is asked without such a card).
    Chosen { pred: Pred },
    /// Up to `max` Energy cards matching the predicate attached to the owner's Benched Pokémon
    /// (a DiscardEnergy prompt over the Bench; nothing is asked without one). `Num::Last` counts the cards.
    FromBench { max: i32, pred: Pred },
    /// An Ability's cost: one card of the Pokémon matching the predicate. With exactly one it is
    /// discarded without asking; with more a cancellable ChooseCards prompt (up to 1) asks. `Num::Last`
    /// counts the cards discarded (0 when declined).
    CostOne { pred: Pred },
    /// A ChooseCards prompt over the Pokémon's cards: between `min` and `max` cards; with `energies_only`
    /// the prompt lists only its Energy cards.
    Cards { min: Num, max: Num, kind: EnergyKind, cancel: bool, energies_only: bool },
    /// A DiscardEnergy prompt over the owner's Active or Benched Pokémon; with `clamp` the numbers
    /// are limited to the Energy cards available.
    Scoped { scope: PromptScope, min: Num, max: Num, kind: EnergyKind, clamp: bool },
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

impl EnergySelection {
    /// The selections that choose with a chooser and a destination (the `ec_*` entries).
    fn is_moved(&self) -> bool {
        matches!(self, EnergySelection::Cards { .. } | EnergySelection::Scoped { .. } | EnergySelection::Cost { .. } | EnergySelection::All { .. } | EnergySelection::Tools { .. } | EnergySelection::ToBench { .. })
    }
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
/// The player chooses one of their Prize cards (face down only when asked); its position is
/// kept for `PrizeVisibility`.
pub struct PickPrizeSpec {
    pub chooser: Who,
    pub face_down_only: bool,
}
/// The resolving Trainer (a Fossil) is played from the hand as a Basic Pokémon on the first
/// open Bench slot; it can't be played without room.
pub struct PlayAsPokemonSpec {}
/// A change to the Prize cards.
pub struct PrizeVisibilitySpec {
    pub action: PrizeAction,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrizeAction {
    /// Redeemable Ticket: the player's Prize cards are shuffled (game RNG, no prompt) and put on
    /// the bottom of the deck, then that many cards from the top of the deck become the Prize
    /// cards, all face down.
    RedealThroughDeck,
    /// Turn the Prize card picked by `PickPrize` face up (visible to both players); the Prize
    /// cards are the given player's.
    FaceUp(Who),
}
/// `who` takes `count` Prize cards into their hand (chosen by them when there are more left).
pub struct TakePrizeSpec {
    pub who: Who,
    pub count: Num,
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
    matches!(z.1, Zone::Scratch(r) if f.cards[r as usize] == NONE) || (matches!(z.1, Zone::Attached(SlotExpr::Picked) | Zone::AttachedEnergy(SlotExpr::Picked)) && f.slot == NONE)
}

/// The cards of a zone in list order; the resolving card is never part of a hand.
fn zone_cards(g: &Game, me: CardId, f: &Frame, z: ZoneRef) -> Vec<CardId> {
    if zone_is_unset(f, z) {
        return Vec::new();
    }
    let mut v = zone_cards_of(g, me, f, z);
    // A Trainer used through an attack stays in the hand it was copied from.
    if z.1 == Zone::Hand && !f.via_attack {
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

/// Does the card satisfy the pick's predicate (and name restriction)?
fn pick_ok(g: &Game, f: &Frame, pick: &PickSpec, c: CardId) -> bool {
    pred(g, c, &pick.predicate)
        && match pick.same_name_as {
            None => true,
            Some(w) => {
                let name = g.st.cdef(c).name;
                in_play(g, f.who(w), PlayScope::All).iter().any(|(_, top, _)| g.st.cdef(*top).name == name)
            }
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
                DrawAmount::UntilHandSize(n) => num_m(g, me, f, n)? - hand_len_without(g, p, me),
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
            if !s.wait {
                shuffle_deck(g, p);
                return Ok(Flow::Next);
            }
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
        Op::PickPrize(pp) => {
            let id = g.player_id(f.who(pp.chooser));
            g.prompt(
                id,
                "CHOOSE_POKEMON",
                PromptKind::ChoosePrize { count: 1, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: false, is_secret: false, destination: None, face_down_only: pp.face_down_only },
                f.cont(me, 1),
            );
            Ok(Flow::Suspend)
        }
        Op::PlayAsPokemon(_) => {
            let p = f.p as usize;
            let Some(s) = empty_bench_slots(g, p).as_slice().first().copied() else { crate::bail!("CANNOT_PLAY_THIS_CARD") };
            // An Item resolving sits in the supporter pile; the Pokémon play takes it from the hand.
            if g.st.players[p].supporter.contains(me) {
                g.move_card_to(ListRef::Supporter(p as u8), me, ListRef::Hand(p as u8));
            }
            g.run_fx(Effect::PlayPokemon { p: p as u8, card: me, target: SlotRef::new(p, s), slot: SlotType::Board, index: 0 })?;
            Ok(Flow::Next)
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
        Op::DiscardEnergy(d) if d.selection.is_moved() => ec_exec(g, me, f, d),
        Op::DiscardEnergy(d) => de_exec(g, me, f, d),
        Op::MoveEnergyOwn(m) => Ok(if move_energy_own_prompt(g, me, f, m) { Flow::Suspend } else { Flow::Next }),
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
        Op::PrizeVisibility(pv) => {
            match pv.action {
                PrizeAction::RedealThroughDeck => redeal_prizes(g, f.p as usize),
                PrizeAction::FaceUp(owner) => {
                    if f.prize != NONE {
                        let (p, i) = (f.who(owner), f.prize as usize);
                        g.st.players[p].prize_public[i] = true;
                        g.st.players[p].prize_face_up[i] = true;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::HandShuffleDraw(h) => {
            let p = f.who(h.who);
            let n = num(g, me, f, &h.draw).max(0) as u8;
            // The resolving card is excluded only from its own player's hand (it is not there when
            // played); a Supporter used through Mr. Mime's attack is still in the opponent's hand
            // and is shuffled in with it (Twinleaf Judge: excludeCard on the player's side only).
            let exclude = if matches!(h.who, Who::Me) { me } else { NO_CARD };
            shuffle_hand_into_deck_then_draw_ex(g, p, exclude, NO_CARD, n, Some((me, f.frame_at(1))))?;
            Ok(Flow::Suspend)
        }
        Op::Order(o) => {
            if zone_is_unset(f, o.zone) {
                return Ok(Flow::Next);
            }
            let list = zone_ref(f, o.zone);
            if g.lst(list).is_empty() {
                return Ok(Flow::Next);
            }
            let id = g.player_id(f.who(o.who));
            g.prompt(id, o.msg, PromptKind::OrderCards { cards: list, allow_cancel: false }, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::BotherBot(b) => bother_exec(g, me, f, b),
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
                    crate::engine::check::take_specific_prizes(g, p, &c.items[..c.len as usize], ListRef::Hand(p as u8), false)?;
                }
                return Ok(Flow::Next);
            }
            tp_begin(g, me, f, t, false)
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
        CardSel::Tools(s) => match slot_of(g, me, f, *s) {
            Some(slot) => (slot.list(), g.st.slot(slot.p as usize, slot.s).tools.iter().collect()),
            None => return Ok(()),
        },
        CardSel::This => match g.st.locate(me) {
            Some(l) => (l, vec![me]),
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
                CardSel::Tools(_) | CardSel::This => unreachable!(),
            };
            match zone_list(g, me, f, m.from, true) {
                Some(l) => (l, cards),
                None => return Ok(()),
            }
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
    let Some(dst) = zone_list(g, me, f, m.to, false) else { return Ok(()) };
    if matches!(m.cards, CardSel::Tools(_)) {
        for c in cards {
            move_cards(g, src, dst, &[c], me)?;
        }
        return Ok(());
    }
    if let Place::AttachTo(e) = m.place {
        let Some(slot) = slot_of(g, me, f, e) else { return Ok(()) };
        return move_cards(g, src, slot.list(), &cards, me);
    }
    match m.place {
        Place::AttachTo(_) => unreachable!(),
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
    let eligible = cards.iter().filter(|c| pick_ok(g, f, pick, **c)).count();
    let hidden = pick.from.1 == Zone::Deck;
    let max = num(g, me, f, &pick.bounds.max).min(max_cap).max(0);
    let min = num(g, me, f, &pick.bounds.min).min(max).max(0);
    if (!hidden && (eligible == 0 || max == 0)) || (hidden && max_cap == 0) {
        return false;
    }
    let mut opts = ChooseCardsOpts::new(to_u8(min), to_u8(max), pick.cancel || cancel);
    for (i, c) in cards.iter().enumerate() {
        if !pick_ok(g, f, pick, *c) {
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
            CapKind::Basic => opts.max_basics = v,
            CapKind::Evolution => opts.max_evolutions = v,
            CapKind::Stage1 => opts.max_stage1 = v,
            CapKind::Stage2 => opts.max_stage2 = v,
        }
    }
    // A hand's prompt lists it without the resolving card.
    let list = if pick.from.1 == Zone::Hand && !f.via_attack && g.lst(zone_ref(f, pick.from)).contains(&me) {
        g.alloc_temp(&cards)
    } else {
        match zone_list(g, me, f, pick.from, false) {
            Some(l) => l,
            None => return false,
        }
    };
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
        SearchDestination::AttachToPicked | SearchDestination::AttachToolToThis => "CHOOSE_CARD_TO_HAND",
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
    let Some(from) = zone_list(g, me, f, s.pick.from, true) else { return Ok(()) };
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
        // The cards wait in the pick's register for the target (`search_attach_prompt`).
        SearchDestination::AttachToPicked => {}
        SearchDestination::AttachToolToThis => {
            let mut bench_idx = 0usize;
            for (i, &b) in g.st.players[p].bench.iter().enumerate() {
                if g.st.slot_pokemon(p, b) == Some(me) {
                    bench_idx = i;
                }
            }
            for c in chosen {
                if g.st.cdef(*c).is_trainer() {
                    let s = g.st.players[p].bench.as_slice()[bench_idx];
                    move_cards(g, from, ListRef::Slot(p as u8, s), &[*c], me)?;
                    g.st.players[p].slots[s as usize].cards.remove(*c);
                    g.st.players[p].slots[s as usize].tools.push(*c);
                }
            }
        }
    }
    Ok(())
}

/// Ask where the searched cards go (resumed at 2); false when no card was chosen.
fn search_attach_prompt(g: &mut Game, me: CardId, f: &Frame, s: &SearchSpec) -> bool {
    if reg_list(g, f, s.pick.into).is_empty() {
        return false;
    }
    let p = f.who(s.pick.chooser);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_ATTACH_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        f.cont(me, 2),
    );
    true
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
    let Some(list) = zone_list(g, me, f, a.from, true) else { return Ok(false) };
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
    if a.different_types {
        o.different_types = true;
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
        Op::Search(s) if f.sub == 2 => {
            // The target of searched cards that are attached: they leave the deck for that Pokémon.
            if let Some(t) = first.slots().first().copied() {
                let cards: Vec<CardId> = reg_list(g, f, s.pick.into).to_vec();
                let from = zone_ref(f, s.pick.from);
                let source_card = g.st.slot_pokemon(f.p as usize, g.st.players[f.p as usize].active).unwrap_or(me);
                move_cards(g, from, t.list(), &cards, source_card)?;
            }
            Ok(Flow::Next)
        }
        Op::Search(s) => {
            let chosen: Vec<CardId> = first.cards().to_vec();
            finish_search(g, me, f, s, &chosen)?;
            if s.destination == SearchDestination::AttachToPicked && search_attach_prompt(g, me, f, s) {
                return Ok(Flow::Suspend);
            }
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
            f.attached_to = NONE;
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
            f.last = ts.len() as i32;
            if ts.is_empty() {
                if a.none_shuffles && !g.st.players[p].deck.is_empty() {
                    open_shuffle(g, me, f, p, 2);
                    return Ok(Flow::Suspend);
                }
                return Ok(Flow::Next);
            }
            let Some(from) = zone_list(g, me, f, a.from, true) else { return Ok(Flow::Next) };
            for (to, c) in ts.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                f.attached_to = encode(target);
                match a.route {
                    AttachRoute::Move | AttachRoute::MovePoisonActive => move_cards(g, from, target.list(), &[c], me)?,
                    AttachRoute::Effect => {
                        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
                    }
                    AttachRoute::MoveShufflePerCard => {
                        move_cards(g, from, target.list(), &[c], me)?;
                        shuffle_deck(g, p);
                    }
                }
                if a.route == AttachRoute::MovePoisonActive && target.p as usize == p && target.s == g.st.players[p].active {
                    crate::engine::phase::add_condition(&mut g.st.players[p].slots[target.s as usize], SpecialCondition::Poisoned);
                }
            }
            Ok(Flow::Next)
        }
        Op::PickPrize(_) => {
            f.prize = match first {
                Res::Prizes(v) => v.get(0).copied().unwrap_or(NONE),
                _ => NONE,
            };
            Ok(Flow::Next)
        }
        Op::MoveEnergyOwn(m) => {
            let p = f.p as usize;
            if let Res::Transfers(ts) = first {
                for (from, to, card) in ts.iter() {
                    let dst = get_target(&g.st, p, *to)?;
                    if let Some(want) = m.to {
                        if slot_of(g, me, f, want).map_or(true, |w| w != dst) {
                            crate::bail!("INVALID_TARGET");
                        }
                    }
                    let src = get_target(&g.st, p, *from)?;
                    move_cards(g, src.list(), dst.list(), &[*card], me)?;
                }
            }
            if m.used_always {
                ability_used(g, p, me);
            }
            Ok(Flow::Next)
        }
        Op::MoveEnergy(m) if m.mode != MoveEnergyMode::Effect => {
            let Res::Transfers(ts) = first else { return Ok(Flow::Next) };
            let p = f.p as usize;
            if let MoveEnergyMode::BenchToActive { ability: true, .. } = m.mode {
                // One source after the other, the chosen cards of each in the order chosen.
                let dst = ListRef::Slot(p as u8, g.st.players[p].active);
                let mut done: Vec<SlotRef> = Vec::new();
                for (from, _, _) in ts.iter() {
                    let src = get_target(&g.st, p, *from)?;
                    if done.contains(&src) {
                        continue;
                    }
                    done.push(src);
                    for (_, _, card) in ts.iter() {
                        if g.lst(src.list()).contains(card) {
                            move_cards(g, src.list(), dst, &[*card], me)?;
                        }
                    }
                }
                return Ok(Flow::Next);
            }
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
        Op::DiscardEnergy(d) if d.selection.is_moved() => ec_resume(g, me, f, d, results, false),
        Op::DiscardEnergy(d) => de_resume(g, me, f, d, first),
        // Resumed after the prefab's draw.
        Op::HandShuffleDraw(_) => Ok(Flow::Next),
        Op::Order(o) => {
            if let Res::Order(ord) = first {
                if let ListRef::Temp(i) = zone_ref(f, o.zone) {
                    crate::game::apply_order(&mut g.temps[i as usize], ord.as_slice());
                }
            }
            Ok(Flow::Next)
        }
        Op::BotherBot(b) => bother_resume(g, me, f, b, first),
        Op::TakePrize(t) => {
            let p = f.who(t.who);
            if let Res::Prizes(ix) = first {
                crate::engine::check::take_specific_prizes(g, p, ix.as_slice(), ListRef::Hand(p as u8), false)?;
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
        Op::DiscardEnergy(d) if d.selection.is_moved() => ec_begin(g, me, f, d, true),
        Op::DiscardEnergy(d) => de_choice(g, me, f, d),
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
        Op::DiscardEnergy(d) if d.selection.is_moved() => ec_resume(g, me, f, d, results, true),
        Op::DiscardEnergy(d) => de_resume_choice(g, me, f, d, first),
        Op::TakePrize(_) => {
            match first {
                Res::Prizes(ix) if !ix.is_empty() => f.record_items(g, me, CHOICE_YES, ix.as_slice()),
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
            DrawAmount::UntilHandSize(n) => num(g, me, f, n) - hand_len_without(g, f.who(d.who), me) > 0,
            // The card's own steps make room to draw (Naveen discards first): its `needs` decide.
            DrawAmount::UntilHandSizeOthers(_) => true,
        },
        Op::Pick(p) => pick_possible(g, me, f, p, i32::MAX),
        Op::Search(s) => pick_possible(g, me, f, &s.pick, search_room(g, f, s)),
        Op::Move(m) => match &m.cards {
            CardSel::Chosen(_) => true,
            CardSel::Tools(_) | CardSel::This => true,
            CardSel::Random(_) | CardSel::All | CardSel::Top(_) | CardSel::Bottom(_) | CardSel::First(_) => !zone_cards(g, me, f, m.from).is_empty() || zone_is_unset(f, m.from),
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
        Op::PlayAsPokemon(_) => !empty_bench_slots(g, f.p as usize).is_empty(),
        Op::MoveEnergyOwn(m) => move_energy_own_possible(g, me, f, m),
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
    let eligible = cards.iter().filter(|c| pick_ok(g, f, pick, **c)).count() as i32;
    let min = num(g, me, f, &pick.bounds.min).max(1).min(room);
    eligible >= min
}

// ---------------------------------------------------------------------------
// S3 appends

/// The Energy a Pokémon provides, as the game checks it.
fn provided_energy(g: &mut Game, slot: SlotRef) -> R<SVec<crate::effects::EnergyEntry, 64>> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: slot.p, source: slot, energy_map: SVec::new() })?;
    Ok(match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    })
}

// DiscardEnergy::Choose

/// Open the ChooseEnergy prompt for `count` Energy of `ty` among the Energy the Pokémon provides.
/// Returns false (no prompt) when there is nothing to pay with.
fn discard_choose_prompt(g: &mut Game, me: CardId, f: &Frame, slot: SlotRef, count: u8, ty: CardType, up_to: bool) -> R<bool> {
    let energy = provided_energy(g, slot)?;
    if energy.is_empty() {
        return Ok(false);
    }
    if ty != ct::COLORLESS && !energy.iter().any(|m| m.provides.iter().any(|t| *t == ty || *t == ct::ANY)) {
        return Ok(false);
    }
    let mut cost = SVec::new();
    let n = if up_to { (count as usize).min(energy.len()) } else { count as usize };
    for _ in 0..n {
        cost.push(ty);
    }
    let id = g.player_id(f.p as usize);
    g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, f.cont(me, 1));
    Ok(true)
}

/// A DiscardCards effect of the attack on the Pokémon.
fn discard_chosen(g: &mut Game, f: &Frame, slot: SlotRef, cards: &[CardId], to_hand: bool) -> R {
    let Some((p, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    let mut cs: SVec<CardId, 64> = SVec::new();
    for c in cards {
        cs.push(*c);
    }
    let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: slot };
    if to_hand {
        g.run_fx(Effect::CardsToHand { b, cards: cs })?;
    } else {
        g.run_fx(Effect::DiscardCards { b, cards: cs })?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MoveEnergy modes other than the attack effect (S3 agent 3)

/// Open the MoveEnergy prompt of a `BenchToActive` move; false when there is nothing to move.
fn move_energy_mode_prompt(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveEnergySpec) -> bool {
    let owner = f.who(m.owner);
    let chooser = f.who(m.chooser);
    let pokemon = for_each_pokemon(g, owner, PlayerType::BottomPlayer);
    let player_type = if owner == chooser { PlayerType::BottomPlayer } else { PlayerType::TopPlayer };
    let mut slots = SVec::new();
    let mut o = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
    let filter = match m.mode {
        MoveEnergyMode::BenchToActive { max, required, ability } => {
            let has = pokemon.iter().any(|(s, _, t)| t.slot == SlotType::Bench && g.st.slot(owner, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()));
            if !has || pokemon.len() <= 1 {
                return false;
            }
            // At least 1, unless the Trainer is used as the effect of an attack.
            o.min = if required && !f.via_attack { 1 } else { 0 };
            o.max = max;
            for (s, _, t) in pokemon.iter() {
                if t.slot == SlotType::Active && ability {
                    o.blocked_from.push(*t);
                } else if t.slot == SlotType::Active {
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
// DiscardEnergy: choosing and discarding

fn energy_cards(g: &Game, slot: SlotRef) -> Vec<CardId> {
    g.st.slot(slot.p as usize, slot.s).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).collect()
}

/// Open the prompt of a choosing selection (resumed at `sub`); false when there is nothing to choose.
fn energy_prompt(g: &mut Game, me: CardId, f: &Frame, d: &DiscardEnergySpec, sub: u8) -> R<bool> {
    let Some(slot) = de_slot(g, me, f, d) else { return Ok(false) };
    let chooser = f.p as usize;
    let id = g.player_id(chooser);
    match &d.selection {
        EnergySelection::Prompt { ty, min, max, .. } => {
            let available = energy_cards(g, slot).len() as i32;
            let (prompt_min, prompt_max) = match ty {
                // An attack's text before the damage: the bounds as written.
                Some(_) => (*min as i32, *max as i32),
                // Limited by what is attached; nothing is asked without Energy.
                None => {
                    if *max == 0 || available == 0 {
                        return Ok(false);
                    }
                    let m = (*max as i32).min(available);
                    ((*min as i32).min(m), m)
                }
            };
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            let mut o = MoveOpts { allow_cancel: false, min: prompt_min as u8, max: Some(prompt_max as u8), ..Default::default() };
            if let Some(t) = ty {
                if let Some(b) = blocked_non_type_energy(g, slot.p as usize, slot.s, *t)? {
                    o.blocked_map.push((CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0), b));
                }
            }
            g.prompt(
                id,
                "CHOOSE_ENERGIES_TO_DISCARD",
                PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                f.cont(me, sub),
            );
            Ok(true)
        }
        EnergySelection::Chosen { pred: pr } => {
            let cards: Vec<CardId> = g.st.slot(slot.p as usize, slot.s).cards.iter().collect();
            if !cards.iter().any(|c| pred(g, *c, pr)) {
                return Ok(false);
            }
            let mut opts = ChooseCardsOpts::new(1, 1, false);
            for (i, c) in cards.iter().enumerate() {
                if !pred(g, *c, pr) {
                    opts.blocked.push(i as u8);
                }
            }
            choose_cards(g, chooser, "CHOOSE_CARD_TO_DISCARD", slot.list(), Filter::none(), opts, f.cont(me, sub));
            Ok(true)
        }
        EnergySelection::FromBench { max, pred: pr } => {
            let p = slot.p as usize;
            let mut available = 0i32;
            let pl = &g.st.players[p];
            for b in pl.bench.iter() {
                available += pl.slots[*b as usize].cards.iter().filter(|c| pred(g, *c, pr)).count() as i32;
            }
            if available == 0 {
                return Ok(false);
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let o = MoveOpts { allow_cancel: false, min: 0, max: Some(available.min(*max) as u8), ..Default::default() };
            g.prompt(
                id,
                "CHOOSE_ENERGIES_TO_DISCARD",
                PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: if matches!(pr, Pred::BasicEnergy) { Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() } } else { Filter::super_type(SuperType::Energy) }, o },
                f.cont(me, sub),
            );
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// The cards a choosing prompt answered.
fn energy_chosen(_f: &Frame, first: Res) -> Vec<CardId> {
    match first {
        Res::CardsFrom(t) => t.iter().map(|(_, c)| *c).collect(),
        Res::Energy(c) => c.iter().copied().collect(),
        other => other.cards().to_vec(),
    }
}

/// One DiscardCards effect per Pokémon the cards are on (first seen first).
fn discard_cards_from_slots(g: &mut Game, me: CardId, f: &Frame, cards: &[CardId]) -> R {
    let attack = if matches!(f.prog, crate::spec::run::Prog::Attack(_)) { attack_data(g, f.eff) } else { None };
    let Some((p, opp, attack, source)) = attack else {
        // An Ability's cost: the cards move to the discard pile.
        for c in cards {
            if let Some(src) = g.st.locate(*c) {
                move_cards(g, src, ListRef::Discard(f.p), &[*c], me)?;
            }
        }
        return Ok(());
    };
    let mut groups: Vec<(SlotRef, SVec<CardId, 64>)> = Vec::new();
    for c in cards {
        let Some(ListRef::Slot(q, s)) = g.st.locate(*c) else { continue };
        let t = SlotRef::new(q as usize, s);
        match groups.iter_mut().find(|(x, _)| *x == t) {
            Some((_, v)) => v.push(*c),
            None => {
                let mut v = SVec::new();
                v.push(*c);
                groups.push((t, v));
            }
        }
    }
    for (target, cards) in groups {
        let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// DiscardEnergy::OppTools (S3 agent 3)

fn opp_tools_prompt(g: &mut Game, me: CardId, f: &Frame, max: u8) -> bool {
    let p = f.p as usize;
    let o = 1 - p;
    let mut tools = 0usize;
    for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        tools += g.st.slot(o, s).tools.len();
    }
    if tools == 0 {
        return false;
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
    let opts = MoveOpts { allow_cancel: false, min: 0, max: Some(tools.min(max as usize) as u8), ..Default::default() };
    let id = g.player_id(p);
    g.prompt(id, "CHOOSE_CARD_TO_DISCARD", PromptKind::DiscardEnergy { player_type: PlayerType::TopPlayer, slots, filter, o: opts }, f.cont(me, 1));
    true
}

/// (slot byte, card) pairs of the answer.
fn opp_tools_items(g: &Game, f: &Frame, ts: &[(CardTarget, CardId)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (from, c) in ts {
        if let Ok(t) = get_target(&g.st, f.p as usize, *from) {
            out.extend_from_slice(&[t.p << 4 | t.s, *c]);
        }
    }
    out
}

fn opp_tools_carry_out(g: &mut Game, me: CardId, f: &Frame, items: &[u8]) -> R {
    let p = f.p as usize;
    let Some((_, _, attack, source)) = attack_data(g, f.eff) else { return Ok(()) };
    let source_card = g.st.slot_pokemon(source.p as usize, source.s).unwrap_or(me);
    for pair in items.chunks(2).filter(|c| c.len() == 2) {
        let t = SlotRef::new((pair[0] >> 4) as usize, pair[0] & 15);
        let owner = t.p as usize;
        // An effect of the attack on that Pokémon: Mist Energy and the like prevent it (ruling 1843).
        if attack_effect_prevented_on(g, p, owner, pack_attack(attack), t)? {
            continue;
        }
        move_cards(g, ListRef::Slot(owner as u8, t.s), ListRef::Discard(owner as u8), &[pair[1]], source_card)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// PrizeVisibility::RedealThroughDeck (S3 agent 3)

fn redeal_prizes(g: &mut Game, p: usize) {
    let pc = g.st.players[p].prize_count as usize;
    let mut all: Vec<CardId> = Vec::new();
    for i in 0..pc {
        all.extend(g.st.players[p].prizes[i].iter());
    }
    let count = all.len();
    let mut perm = [0u8; 120];
    g.rng.shuffle(all.len(), &mut perm);
    let copy = all.clone();
    for i in 0..all.len() {
        all[i] = copy[perm[i] as usize];
    }
    // Each goes onto the bottom of the deck, in that order.
    let mut deck: Vec<CardId> = g.st.players[p].deck.iter().collect();
    deck.extend(all.iter().copied());
    for i in 0..pc {
        g.st.players[p].prizes[i].set_from(&[]);
    }
    // The new Prizes come from the top of the deck, into the first empty Prize slots.
    for _ in 0..count {
        if deck.is_empty() {
            continue;
        }
        let c = deck.remove(0);
        match (0..pc).find(|i| g.st.players[p].prizes[*i].is_empty()) {
            Some(i) => g.st.players[p].prizes[i].set_from(&[c]),
            None => deck.insert(0, c),
        }
    }
    g.st.players[p].deck.set_from(&deck);
    // The new Prizes are face down.
    g.st.players[p].prize_public = [false; 6];
    g.st.players[p].prize_face_up = [false; 6];
}

/// Ability: move an Energy from one of your Pokémon to another (or to `to`).
pub struct MoveEnergyOwnSpec {
    /// The Pokémon the Energy must move to (None: any of your Pokémon).
    pub to: Option<SlotExpr>,
    /// Which Energy cards can move.
    pub energy: Pred,
    /// "May": the prompt can be answered with nothing.
    pub cancel: bool,
    /// The Ability counts as used even when nothing moved.
    pub used_always: bool,
}

fn move_energy_own_possible(g: &Game, me: CardId, f: &Frame, m: &MoveEnergyOwnSpec) -> bool {
    let p = f.p as usize;
    let dest = m.to.and_then(|e| slot_of(g, me, f, e));
    let pokemon = for_each_pokemon(g, p, PlayerType::BottomPlayer);
    if dest.is_none() && pokemon.len() <= 1 {
        return false;
    }
    pokemon.iter().any(|(s, _, _)| dest.map_or(true, |d| d.s != *s) && g.st.slot(p, *s).cards.iter().any(|c| pred(g, c, &m.energy)))
}

fn move_energy_own_prompt(g: &mut Game, me: CardId, f: &mut Frame, m: &MoveEnergyOwnSpec) -> bool {
    if !move_energy_own_possible(g, me, f, m) {
        return false;
    }
    let p = f.p as usize;
    let mut o = MoveOpts { allow_cancel: m.cancel, min: if m.cancel { 0 } else { 1 }, max: Some(1), ..Default::default() };
    if let Some(dest) = m.to.and_then(|e| slot_of(g, me, f, e)) {
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if s == dest.s {
                o.blocked_from.push(t);
            } else {
                o.blocked_to.push(t);
            }
        }
    }
    let mut filter = Filter::super_type(SuperType::Energy);
    match &m.energy {
        Pred::BasicEnergy => filter.energy_type = Some(EnergyType::Basic as u8),
        Pred::All(ps) if ps.iter().any(|q| matches!(q, Pred::BasicEnergy)) => {
            filter.energy_type = Some(EnergyType::Basic as u8);
            for q in ps.iter() {
                if let Pred::Name(n) = q {
                    filter.name = Some(n);
                }
            }
        }
        _ => {}
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, f.cont(me, 1));
    true
}

/// "Discard a Stadium in play": the Stadium (only one is in play) goes to its owner's discard pile.
pub const DISCARD_STADIUM: Op = Op::If(IfSpec {
    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Stadium), Pred::Any),
    yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Stadium), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
    no: &[Step::new(Op::If(IfSpec {
        cond: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Stadium), Pred::Any),
        yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Stadium), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
        no: &[],
    }))],
});

// ---------------------------------------------------------------------------
// DiscardEnergy: one entry per phase

/// Cards of the Pokémon go back to their owner's hand as an effect of the attack.
fn to_hand_of(g: &mut Game, f: &Frame, slot: SlotRef, cards: &[CardId]) -> R {
    discard_chosen(g, f, slot, cards, true)
}

/// The fixed Pokémon of a selection that does not choose one.
fn de_slot(g: &Game, me: CardId, f: &Frame, d: &DiscardEnergySpec) -> Option<SlotRef> {
    match &d.target {
        SlotTarget::Slot(x) => slot_of(g, me, f, *x),
        SlotTarget::Pick(_) => None,
    }
}

fn de_exec(g: &mut Game, me: CardId, f: &mut Frame, d: &DiscardEnergySpec) -> R<Flow> {
    // The choice made at step D, carried out now.
    if let Some(c) = f.recorded_choice(g, me) {
        if c.answer == CHOICE_YES {
            de_carry_out(g, me, f, d, &c.items[..c.len as usize])?;
        }
        return Ok(Flow::Next);
    }
    let Some(slot) = de_slot(g, me, f, d) else { return Ok(Flow::Next) };
    match &d.selection {
        EnergySelection::AllProvided | EnergySelection::Provides(_) => {
            let map = provided_energy(g, slot)?;
            let mut cards: Vec<CardId> = Vec::new();
            for m in map.iter() {
                let hit = match &d.selection {
                    EnergySelection::Provides(t) => m.provides.iter().any(|x| *x == *t || *x == ct::ANY),
                    _ => true,
                };
                if hit && !cards.contains(&m.card) {
                    cards.push(m.card);
                }
            }
            discard_cards_from_slots(g, me, f, &cards)?;
        }
        EnergySelection::Register(r) => {
            let cards: Vec<CardId> = reg_list(g, f, *r).to_vec();
            if !cards.is_empty() {
                discard_cards_from_slots(g, me, f, &cards)?;
            }
        }
        EnergySelection::Special => {
            let cards: Vec<CardId> = g.st.slot(slot.p as usize, slot.s).cards.iter().filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Special as u8
            }).collect();
            if !cards.is_empty() {
                discard_cards_from_slots(g, me, f, &cards)?;
            }
        }
        EnergySelection::AllIntoDeck => {
            let (p, s) = (slot.p as usize, slot.s);
            let energies: Vec<CardId> = g.st.slot(p, s).energies.iter().collect();
            if !energies.is_empty() {
                move_cards_after_damage(g, f.eff, ListRef::Slot(p as u8, s), ListRef::Deck(p as u8), &energies, me)?;
            }
            shuffle_deck_after_damage(g, f.eff, p);
        }
        EnergySelection::Among(a) => return among_exec(g, me, f, slot, a),
        EnergySelection::OppTools { max } => return Ok(if opp_tools_prompt(g, me, f, *max) { Flow::Suspend } else { Flow::Next }),
        EnergySelection::Choose { count, ty } => {
            return Ok(if discard_choose_prompt(g, me, f, slot, *count, *ty, false)? { Flow::Suspend } else { Flow::Next });
        }
        EnergySelection::ChooseToHand { count, ty, up_to } => {
            return Ok(if discard_choose_prompt(g, me, f, slot, *count, *ty, *up_to)? { Flow::Suspend } else { Flow::Next });
        }
        EnergySelection::Prompt { .. } | EnergySelection::Chosen { .. } | EnergySelection::FromBench { .. } => {
            return Ok(if energy_prompt(g, me, f, d, 1)? { Flow::Suspend } else { Flow::Next });
        }
        EnergySelection::CostOne { pred: pr } => {
            let cards: Vec<CardId> = g.st.slot(slot.p as usize, slot.s).cards.iter().collect();
            let matching: Vec<CardId> = cards.iter().copied().filter(|c| pred(g, *c, pr)).collect();
            f.last = 0;
            match matching.len() {
                0 => {}
                1 => {
                    discard_cards_from_slots(g, me, f, &matching)?;
                    f.last = 1;
                }
                _ => {
                    let mut opts = ChooseCardsOpts::new(0, 1, true);
                    for (i, c) in cards.iter().enumerate() {
                        if !pred(g, *c, pr) {
                            opts.blocked.push(i as u8);
                        }
                    }
                    choose_cards(g, f.p as usize, "CHOOSE_CARD_TO_DISCARD", slot.list(), Filter::none(), opts, f.cont(me, 1));
                    return Ok(Flow::Suspend);
                }
            }
        }
        _ => unreachable!("moved selections run in the ec_* entries"),
    }
    Ok(Flow::Next)
}

/// Carry out an answer (the chosen cards, or slot and card pairs for `OppTools`).
fn de_carry_out(g: &mut Game, me: CardId, f: &mut Frame, d: &DiscardEnergySpec, items: &[u8]) -> R {
    match &d.selection {
        EnergySelection::OppTools { .. } => opp_tools_carry_out(g, me, f, items),
        EnergySelection::ChooseToHand { .. } => {
            if let Some(slot) = de_slot(g, me, f, d) {
                to_hand_of(g, f, slot, items)?;
            }
            Ok(())
        }
        _ => {
            discard_cards_from_slots(g, me, f, items)?;
            f.last = items.len() as i32;
            Ok(())
        }
    }
}

fn de_resume(g: &mut Game, me: CardId, f: &mut Frame, d: &DiscardEnergySpec, first: Res) -> R<Flow> {
    match &d.selection {
        EnergySelection::Among(a) => among_resume(g, me, f, a, first)?,
        EnergySelection::OppTools { .. } => {
            if let Res::CardsFrom(ts) = first {
                let items = opp_tools_items(g, f, ts.as_slice());
                opp_tools_carry_out(g, me, f, &items)?;
            }
        }
        EnergySelection::Prompt { into, .. } => {
            let cards = energy_chosen(f, first);
            if let Some(r) = into {
                set_reg(g, f, *r, &cards);
            }
            de_carry_out(g, me, f, d, &cards)?;
        }
        _ => {
            let cards = energy_chosen(f, first);
            de_carry_out(g, me, f, d, &cards)?;
        }
    }
    Ok(Flow::Next)
}

/// Step D: ask now what the effect will discard after the damage.
fn de_choice(g: &mut Game, me: CardId, f: &mut Frame, d: &DiscardEnergySpec) -> R<Flow> {
    let asked = match &d.selection {
        EnergySelection::OppTools { max } => opp_tools_prompt(g, me, f, *max),
        EnergySelection::Choose { count, ty } => match de_slot(g, me, f, d) {
            Some(slot) => discard_choose_prompt(g, me, f, slot, *count, *ty, false)?,
            None => return Ok(Flow::Next),
        },
        EnergySelection::ChooseToHand { count, ty, up_to } => match de_slot(g, me, f, d) {
            Some(slot) => discard_choose_prompt(g, me, f, slot, *count, *ty, *up_to)?,
            None => return Ok(Flow::Next),
        },
        EnergySelection::Prompt { ty: None, .. } | EnergySelection::Chosen { .. } | EnergySelection::FromBench { .. } => energy_prompt(g, me, f, d, 1)?,
        _ => return Ok(Flow::Next),
    };
    if asked {
        return Ok(Flow::Suspend);
    }
    f.record(g, me, CHOICE_NONE);
    Ok(Flow::Next)
}

fn de_resume_choice(g: &mut Game, me: CardId, f: &mut Frame, d: &DiscardEnergySpec, first: Res) -> R<Flow> {
    if let EnergySelection::OppTools { .. } = d.selection {
        match first {
            Res::CardsFrom(ts) => {
                let items = opp_tools_items(g, f, ts.as_slice());
                f.record_items(g, me, CHOICE_YES, &items);
            }
            _ => f.record(g, me, CHOICE_NONE),
        }
        return Ok(Flow::Next);
    }
    let cards = energy_chosen(f, first);
    if cards.is_empty() {
        f.record(g, me, CHOICE_NONE);
    } else {
        f.record_items(g, me, CHOICE_YES, &cards);
    }
    Ok(Flow::Next)
}

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
fn ec_owner(f: &Frame, e: &DiscardEnergySpec) -> usize {
    match &e.target {
        SlotTarget::Slot(SlotExpr::Active(w)) => f.who(*w),
        _ => f.p as usize,
    }
}

fn ec_ptype(g: &Game, f: &Frame, e: &DiscardEnergySpec, owner: usize) -> PlayerType {
    let _ = g;
    if owner == f.who(e.chooser) {
        PlayerType::BottomPlayer
    } else {
        PlayerType::TopPlayer
    }
}

fn ec_begin(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, record: bool) -> R<Flow> {
    let none = |g: &mut Game, f: &mut Frame| {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        Ok(Flow::Next)
    };
    if !cond_m(g, me, f, &e.when)? {
        return none(g, f);
    }
    if let EnergySelection::Scoped { .. } = &e.selection {
        return ec_prompt(g, me, f, e, record);
    }
    if let EnergySelection::Tools { .. } = &e.selection {
        return ec_tools(g, me, f, e, record);
    }
    let src = match &e.target {
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
fn ec_stage2(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, src: SlotRef, record: bool) -> R<Flow> {
    let chooser = f.who(e.chooser);
    let sub = 0x80 | super::board::encode(src);
    let none = |g: &mut Game, f: &mut Frame| {
        if record {
            f.record(g, me, CHOICE_NONE);
        }
        Ok(Flow::Next)
    };
    let (p, s) = (src.p as usize, src.s);
    match &e.selection {
        EnergySelection::Cards { min, max, kind, cancel, energies_only } => {
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
        EnergySelection::Cost { n, ty } => {
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
        EnergySelection::All { provided } => {
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
        EnergySelection::ToBench { min, max, same_target, .. } => {
            let bench: Vec<SlotRef> = slots_of(g, me, f, &SlotSel::Bench(if p == f.p as usize { Who::Me } else { Who::Opp })).iter().copied().filter(|b| *b != src).collect();
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
            // A Benched source (the attacker switched itself out) is not a place to move to.
            if let Some(i) = g.st.players[p].bench.iter().position(|b| *b == s) {
                o.blocked_to.push(CardTarget::new(ptype, SlotType::Bench, i as u8));
            }
            let id = g.player_id(chooser);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, s), player_type: ptype, slots, filter: Filter::super_type(SuperType::Energy), o },
                f.cont(me, sub),
            );
            Ok(Flow::Suspend)
        }
        _ => unreachable!("handled by ec_prompt / ec_tools or the DiscardEnergy entries"),
    }
}

/// A DiscardEnergy prompt over the Active or Benched Pokémon of the owner.
fn ec_prompt(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, record: bool) -> R<Flow> {
    let EnergySelection::Scoped { scope, min, max, kind, clamp } = &e.selection else { return Ok(Flow::Next) };
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

fn ec_exec(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec) -> R<Flow> {
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
fn ec_resume(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, results: &[Res], record: bool) -> R<Flow> {
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
    match (&e.selection, first) {
        (EnergySelection::Cards { .. }, Res::Cards(c)) => {
            if let Some(src) = src {
                ts.extend(c.as_slice().iter().map(|x| (src, None, *x)));
            }
        }
        (EnergySelection::Cost { .. }, Res::Energy(c)) => {
            if let Some(src) = src {
                ts.extend(c.as_slice().iter().map(|x| (src, None, *x)));
            }
        }
        (EnergySelection::Scoped { .. } | EnergySelection::Tools { .. }, Res::CardsFrom(t)) => {
            for (from, c) in t.iter().copied() {
                if let Ok(slot) = get_target(&g.st, chooser, from) {
                    ts.push((slot, None, c));
                }
            }
        }
        (EnergySelection::ToBench { .. }, Res::Attach(t)) => {
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
fn ec_apply(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, ts: &[Ec]) -> R {
    let tools = matches!(e.selection, EnergySelection::Tools { .. });
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
        match (&e.selection, e.to) {
            (EnergySelection::ToBench { via_effect, .. }, _) => {
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
fn ec_tools(g: &mut Game, me: CardId, f: &mut Frame, e: &DiscardEnergySpec, record: bool) -> R<Flow> {
    let EnergySelection::Tools { min, max } = &e.selection else { return Ok(Flow::Next) };
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

// Team Rocket's Bother-Bot

/// Key of the side copy of the chosen Prize index and hand card across the prompts.
const BOTHER_STASH: u64 = u64::MAX - 1;

fn bother_show(g: &mut Game, me: CardId, f: &Frame, sub: u8) {
    let id = g.player_id(f.p as usize);
    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, f.cont(me, sub));
}

fn bother_exec(g: &mut Game, me: CardId, f: &mut Frame, pp: &BotherBotSpec) -> R<Flow> {
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

fn bother_resume(g: &mut Game, me: CardId, f: &mut Frame, pp: &BotherBotSpec, first: Res) -> R<Flow> {
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
            let mut ch = SpecChoice { card: me, key: BOTHER_STASH, answer: 0, items: [0; SPEC_CHOICE_ITEMS], len: 2 };
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

/// Team Rocket's Bother-Bot: turn 1 of `whose`'s face-down Prize cards face up (it stays face up), look at a
/// random card of their hand, and you may have them switch those cards.
pub struct BotherBotSpec {
    pub whose: Who,
}
