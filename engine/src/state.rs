//! Game state: plain `Copy` data, no heap. Cloning is a memcpy.
//!
//! Mirrors Twinleaf's `State` / `Player` / `PokemonCardList` / `CardList`,
//! keeping only fields that rules or pool cards read or write. Pokémon slots
//! live in a per-player arena so a slot keeps its identity when it moves
//! between the Active Spot and the Bench (Twinleaf swaps the objects).

use crate::carddb::{def, CardDef, DefId, PF_FOSSIL, PF_POKEMON};
use crate::list::*;
use crate::markers::*;
use crate::types::*;

pub const MAX_CARDS: usize = 120;
pub const MAX_SLOTS: usize = 9;
/// Pokémon references from both players at once (damaged lists, multi-Pokémon answers).
pub const MAX_SLOT_REFS: usize = 2 * MAX_SLOTS;
pub const MAX_BENCH: usize = 8;

/// Zone lists hold up to 120: Twinleaf can duplicate cards (energies of a
/// fully moved Pokémon slot are pushed twice), so zones may exceed 60.
pub type Deck = ZoneList<120>;
pub type SlotId = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MarkerItem {
    pub name: MarkerName,
    pub source: CardId,
    pub source_type: SourceType,
    pub target_scope: TargetScope,
}

/// `Marker`: ordered list of marker items. Slots hold up to 12; a player's
/// list holds up to 48 ([`PlayerMarker`]): many cards add a marker per copy
/// in either deck (Team Rocket's Archer: one per Archer, 8 in a mirror match).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Marker<const N: usize = 12> {
    pub items: SVec<MarkerItem, N>,
}

pub type PlayerMarker = Marker<48>;

impl<const N: usize> Default for Marker<N> {
    fn default() -> Self {
        Marker { items: SVec::new() }
    }
}

impl<const N: usize> Marker<N> {
    pub fn has(&self, name: MarkerName) -> bool {
        self.items.iter().any(|m| m.name == name)
    }
    pub fn has_from(&self, name: MarkerName, source: CardId) -> bool {
        self.items.iter().any(|m| m.name == name && m.source == source)
    }
    pub fn remove(&mut self, name: MarkerName) {
        self.items.retain(|m| m.name != name);
    }
    pub fn remove_from(&mut self, name: MarkerName, source: CardId) {
        self.items.retain(|m| !(m.source == source && m.name == name));
    }
    pub fn add(&mut self, name: MarkerName, source: CardId, st: SourceType, scope: TargetScope) {
        if self.has_from(name, source) {
            return;
        }
        self.items.push(MarkerItem { name, source, source_type: st, target_scope: scope });
    }
    /// `addMarkerToState`: sourceless marker, once.
    pub fn add_to_state(&mut self, name: MarkerName) {
        if self.has(name) {
            return;
        }
        self.items.push(MarkerItem { name, source: NO_CARD, ..Default::default() });
    }
    pub fn remove_attack_effects(&mut self) {
        self.items.retain(|m| m.source_type != SourceType::Attack);
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }
    /// `removeAllExceptTrainerEffects`: keeps the markers a Trainer card put on a Pokémon when it moves to the Bench
    /// (Acerola's Mischief, id2228; an Ability's effect too, id1651). Evolving and devolving end them all
    /// (`game_effect::clear_effects_evolving`).
    pub fn remove_all_except_trainer_effects(&mut self) {
        self.items.retain(|m| m.source_type == SourceType::Trainer);
    }
}

/// `PokemonCardList`: one board slot.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub cards: ZoneList<60>,
    pub energies: ZoneList<60>,
    pub tools: ZoneList<4>,
    pub damage: i32,
    /// Order matters (between-turns processing iterates it).
    pub special_conditions: SVec<u8, 5>,
    pub poison_damage: i32,
    pub burn_damage: i32,
    pub confusion_damage: i32,
    pub marker: Marker,
    /// The turn the Pokémon in this spot came into play: put into play, evolved or devolved (APR A-05,
    /// C-13; id2229). Slot-bound (events design 4.5). No permission rewrites it: permissions lift limits
    /// (`Modifier::Permit`) instead. The one record of the fact: evolution timing, "played this turn"
    /// (`SlotPred::PlayedThisTurn`), the observation and the canonical JSON (`pokemonPlayedTurn`) read it. The
    /// Pokémon set up before the first turn count as entering on their owner's first turn (1 or 2,
    /// `setup::finish`).
    pub entered_turn: i32,
    pub sleep_flips: i32,
    pub board_effect: SVec<u8, 6>,
    /// `attacksThisTurn` (absent until first written).
    pub attacks_this_turn: Option<i32>,
    pub healed_this_turn: bool,
    /// An effect Knocked this Pokémon Out ("is Knocked Out", "Knock Out ..."; `engine::knockout::by_effect`): it is
    /// Knocked Out at the next state check with every other Knock Out (user decision D1), by this cause.
    pub ko_pending: Option<crate::cause::Cause>,
    pub is_public: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            cards: List::new(),
            energies: List::new(),
            tools: List::new(),
            damage: 0,
            special_conditions: SVec::new(),
            poison_damage: 10,
            burn_damage: 20,
            confusion_damage: 30,
            marker: Marker::default(),
            entered_turn: 0,
            sleep_flips: 1,
            board_effect: SVec::new(),
            attacks_this_turn: None,
            healed_this_turn: false,
            ko_pending: None,
            is_public: false,
        }
    }
}


/// Per-instance card data that changes during a game.
#[derive(Clone, Copy, Debug)]
pub struct CardInst {
    pub def: DefId,
    pub owner: u8,
    pub moved_to_active_this_turn: bool,
    pub damage_taken_last_turn: i32,
    /// Take-hold stamp of an Ability lock (docs/rulings/RULES.md, precedence between locks): the order in which
    /// the lock's source took hold, 0 while it doesn't hold. Kept up to date by `lock_sync`.
    pub lock_stamp: u16,
    /// The rules zone the card was in when it was moved into a staging list (`Game::temps`: cards looked at
    /// or searched for), so an event that takes it from there knows its source (`engine::enter::source_of`).
    pub staged_from: Option<crate::spec::event::RulesZone>,
    /// The staging list the card was last moved into (`ListRef::Temp` index): where it physically is while it
    /// sits there. A card register can hold a copy of it too (a Pick's selection); `source_of` takes the card
    /// from this list, never from a copy.
    pub staged_in: Option<u8>,
}

impl Default for CardInst {
    fn default() -> Self {
        CardInst { def: 0, owner: 0, moved_to_active_this_turn: false, damage_taken_last_turn: 0, lock_stamp: 0, staged_from: None, staged_in: None }
    }
}

/// Whom a lasting effect is on: a Pokémon (the spot it is in: the effects of attacks stay with the Pokémon in its spot,
/// APR C-15) or a player (APR C-19).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LastingOn {
    Pokemon(u8, SlotId),
    Player(u8),
}

impl LastingOn {
    pub fn pokemon(p: usize, s: SlotId) -> LastingOn {
        LastingOn::Pokemon(p as u8, s)
    }
}

/// A lasting effect an attack left ("during your opponent's next turn, ...", "this Pokémon can't attack during your next
/// turn"; the ApplyEffect event's reducer puts it, `engine::apply`): what it is, on whom, and the card whose attack left
/// it. The lasting table (`State::lasting`, events batch 8 C7) holds every one; `engine::lasting` reads it, rolls its
/// timing over at the end of each turn and removes a Pokémon's with its other effects (APR C-13, C-15).
#[derive(Clone, Copy, Debug)]
pub struct LastingEffect {
    pub on: LastingOn,
    /// The card whose attack left it (a lock's and a prevention's predicates are evaluated for it; the retaliating
    /// Pokémon; the Pokémon the next-turn bonus is for).
    pub source: CardId,
    pub what: LastingWhat,
}

/// What a lasting effect does, with the state of its timing ("pending": put during this turn, in force from the end of
/// it; "armed": in force).
#[derive(Clone, Copy, Debug)]
pub enum LastingWhat {
    /// "This Pokémon can't attack / retreat / use [Attack Name]" (APR C-15; events batch 7, D14): the declared lock, the
    /// turns of the locked player it still lasts ([`LastingWhat::UNTIL_REMOVED`]: until the Pokémon's effects end), and
    /// the attack it names (its name is the one forbidden).
    Lock { decl: &'static crate::spec::passive::LockDecl, turns: i8, attack: Option<AttackRef> },
    /// "Your opponent can't ..." (APR C-19; Walrein's "Pokémon that have 2 or less Energy attached can't attack"): the
    /// declared lock, the turns it still lasts, and its place among the player's (the order they are read in).
    PlayerLock { decl: &'static crate::spec::passive::LockDecl, turns: i8, pos: u8 },
    /// "This Pokémon takes N less damage from attacks."
    TakesLessDamage(i16),
    /// "Attacks used by the Defending Pokémon do N less damage."
    DealsLessDamage(i16),
    /// "The Defending Pokémon takes N more damage from attacks" (by the attacking player's attacks).
    TakesMoreDamage { amount: i16, attacker: Option<u8>, pending: bool, rearm: bool },
    /// "Prevent all damage done to / effects of attacks done to this Pokémon": a `Prevent` (an index into
    /// `passive::LASTING_PREVENTS`), evaluated for the source card.
    Prevent { spec: u8, pending: bool },
    /// A lasting trigger: "if this Pokémon is damaged by an attack, put N damage counters on the Attacking Pokémon"
    /// (the source's attack `attack`, by player `attacker`).
    Retaliate { amount: i16, attack: AttackRef, attacker: u8, pending: bool },
    /// A lasting trigger: "if this Pokémon is Knocked Out by damage from an attack, discard an Energy from the Attacking
    /// Pokémon" (Little Grudge; the source's attack `attack`, by player `attacker`).
    DiscardAttackerEnergy { attack: AttackRef, attacker: u8, armed: bool, pending: bool },
    /// "Attacks used by the Defending Pokémon cost [C] more" (`now` in force, `pending` put this turn).
    AttackCostIncrease { now: i16, pending: i16, attacker: Option<u8> },
    /// "The Defending Pokémon's Retreat Cost is [C] more".
    RetreatCostIncrease { now: i16, pending: i16, attacker: Option<u8> },
    /// "This Pokémon has no Weakness."
    NoWeakness { armed: bool, pending: bool },
    /// "During your next turn, [this attack] does N more damage" (a `NextTurnBonus` declaration of the source): the
    /// bonus is for the source card's use of `attack`.
    NextTurnBonus { spec: &'static crate::spec::passive::NextTurnBonusSpec, attack: AttackRef, pending: bool },
    /// "(This includes new Pokémon that come into play.)": on a player, the effect `spec` puts on each new Pokémon of
    /// theirs that comes into play while it lasts (`Op::ArmEach`), with the cause and attack that put it; it ends at the end
    /// of player `ends_with`'s turn.
    /// (`cause` packed, `Cause::pack`: the entry stays within the table's 24 B.)
    Spawn { spec: &'static crate::spec::ops::state::ArmEachSpec, cause: [u8; 4], ends_with: u8 },
}

impl LastingWhat {
    /// The turns of a lock that lasts until the Pokémon's effects end ("until it leaves the Active Spot").
    pub const UNTIL_REMOVED: i8 = i8::MAX;

    /// Nothing in force nor pending: the entry is removed.
    pub fn is_spent(&self) -> bool {
        match *self {
            LastingWhat::Lock { .. } | LastingWhat::PlayerLock { .. } | LastingWhat::Prevent { .. } | LastingWhat::Retaliate { .. } | LastingWhat::NextTurnBonus { .. } | LastingWhat::DiscardAttackerEnergy { .. } | LastingWhat::Spawn { .. } => false,
            LastingWhat::TakesLessDamage(n) | LastingWhat::DealsLessDamage(n) => n == 0,
            LastingWhat::TakesMoreDamage { amount, attacker, pending, rearm } => amount == 0 && attacker.is_none() && !pending && !rearm,
            LastingWhat::AttackCostIncrease { now, pending, attacker } | LastingWhat::RetreatCostIncrease { now, pending, attacker } => now == 0 && pending == 0 && attacker.is_none(),
            LastingWhat::NoWeakness { armed, pending } => !armed && !pending,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    /// Twinleaf client id (1 or 2).
    pub id: u8,
    pub deck: Deck,
    pub hand: Deck,
    pub discard: Deck,
    pub lostzone: Deck,
    pub stadium: ZoneList<4>,
    pub supporter: ZoneList<8>,
    pub prizes: [ZoneList<4>; 6],
    pub prize_count: u8,
    /// Prize `CardList.isSecret == false` (Cresselia SFA turns one public);
    /// all prize lists start secret.
    pub prize_public: [bool; 6],
    /// Prize `CardList.faceUpPrize` (canonical `faceUpPrizes`).
    pub prize_face_up: [bool; 6],
    pub slots: [Slot; MAX_SLOTS],
    pub slot_used: [bool; MAX_SLOTS],
    pub active: SlotId,
    pub bench: SVec<SlotId, MAX_BENCH>,

    /// The Trainer cards the player played from their hand this turn (events batch 7, the PlayTrainer event's record;
    /// cleared at the end of their turn): the one source of "if you played a Supporter card from your hand this turn"
    /// (APR E-26) and of the turn's limits (one Supporter, one Stadium). A used card (Look-Alike Show) isn't played.
    pub played_this_turn: SVec<CardId, 60>,
    pub retreated_turn: i32,
    pub energy_played_turn: i32,
    pub stadium_used_turn: i32,
    pub marker: PlayerMarker,
    pub used_vstar: bool,
    pub used_gx: bool,
    pub prizes_taken: i32,
    pub prizes_taken_this_turn: i32,
    pub prizes_taken_last_turn: i32,
    pub moved_to_active_this_turn: SVec<CardId, 64>,
    pub moved_from_active_to_bench_this_turn: SVec<CardId, 64>,
    pub pokemon_knocked_out_during_opponents_last_turn: bool,
    pub pokemon_knocked_out_by_attack_during_opponents_last_turn: bool,
    pub pokemon_knocked_out_last_turn_entries: SVec<DefId, 8>,
    /// Parallel to the entries: whether that Pokémon was Knocked Out by damage from an attack.
    pub pokemon_knocked_out_last_turn_by_attack: SVec<bool, 8>,
    /// The cards whose declaration says "once per game" (`Once::PerGame`) and has been applied for this player: by the
    /// declaring card's def, never reset. Task C6 of events batch 9 generalizes the usage records into one table.
    pub once_per_game_used: SVec<DefId, 4>,
}

impl Player {
    /// `createPlayer`: 6 face-down prize lists, 5 empty bench slots.
    pub fn new(id: u8) -> Player {
        let mut p = Player {
            id,
            deck: List::new(),
            hand: List::new(),
            discard: List::new(),
            lostzone: List::new(),
            stadium: List::new(),
            supporter: List::new(),
            prizes: [List::new(); 6],
            prize_count: 6,
            prize_public: [false; 6],
            prize_face_up: [false; 6],
            slots: [Slot::default(); MAX_SLOTS],
            slot_used: [false; MAX_SLOTS],
            active: 0,
            bench: SVec::new(),
            played_this_turn: SVec::new(),
            retreated_turn: 0,
            energy_played_turn: 0,
            stadium_used_turn: 0,
            marker: Marker::default(),
            used_vstar: false,
            used_gx: false,
            prizes_taken: 0,
            prizes_taken_this_turn: 0,
            prizes_taken_last_turn: 0,
            moved_to_active_this_turn: SVec::new(),
            moved_from_active_to_bench_this_turn: SVec::new(),
            pokemon_knocked_out_during_opponents_last_turn: false,
            pokemon_knocked_out_by_attack_during_opponents_last_turn: false,
            pokemon_knocked_out_last_turn_entries: SVec::new(),
            pokemon_knocked_out_last_turn_by_attack: SVec::new(),
            once_per_game_used: SVec::new(),
        };
        p.slot_used[0] = true;
        p.slots[0].is_public = true;
        for i in 1..=5 {
            p.slot_used[i] = true;
            p.slots[i].is_public = true;
            p.bench.push(i as SlotId);
        }
        p
    }

    pub fn active_slot(&self) -> &Slot {
        &self.slots[self.active as usize]
    }
    pub fn active_slot_mut(&mut self) -> &mut Slot {
        &mut self.slots[self.active as usize]
    }
    pub fn bench_slot(&self, i: usize) -> &Slot {
        &self.slots[self.bench.as_slice()[i] as usize]
    }
    /// `forEachPokemon` order: active, then bench in order (occupied only).
    pub fn in_play(&self) -> SVec<SlotId, MAX_SLOTS> {
        let mut out = SVec::new();
        if !self.slots[self.active as usize].cards.is_empty() {
            out.push(self.active);
        }
        for &b in self.bench.iter() {
            if !self.slots[b as usize].cards.is_empty() {
                out.push(b);
            }
        }
        out
    }
    /// All board slots (active + bench), occupied or not.
    pub fn all_slots(&self) -> SVec<SlotId, MAX_SLOTS> {
        let mut out = SVec::new();
        out.push(self.active);
        for &b in self.bench.iter() {
            out.push(b);
        }
        out
    }
    pub fn bench_index_of(&self, s: SlotId) -> Option<usize> {
        self.bench.position(&s)
    }
    pub fn prize_left(&self) -> usize {
        self.prizes.iter().map(|p| p.len()).sum()
    }
    /// Allocate a fresh (empty) slot in the arena.
    pub fn alloc_slot(&mut self) -> SlotId {
        for i in 0..MAX_SLOTS {
            if !self.slot_used[i] {
                self.slot_used[i] = true;
                self.slots[i] = Slot::default();
                return i as SlotId;
            }
        }
        panic!("slot arena exhausted");
    }
    pub fn free_slot(&mut self, s: SlotId) {
        self.slot_used[s as usize] = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackRef {
    /// Card instance that owns the attack object.
    pub card: CardId,
    /// Attack index; with [`AttackRef::CLONE`] set it is a copy-attack
    /// clone (`cloneAttacks`, see `copy_attack.rs`): bits 4..6 hold the
    /// session serial, bits 0..3 the index into `card`'s attacks.
    pub index: u8,
}

impl AttackRef {
    pub const CLONE: u8 = 0x80;
    /// Index into the owning card's printed attacks.
    #[inline]
    pub fn idx(self) -> usize {
        (self.index & 0x0F) as usize
    }
    #[inline]
    pub fn is_clone(self) -> bool {
        self.index & Self::CLONE != 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub phase: GamePhase,
    pub turn: i32,
    pub active_player: u8,
    pub winner: Winner,
    /// Why the game ended (`engine::knockout::game_end`).
    pub end_reason: Option<crate::engine::knockout::EndReason>,
    pub players: [Player; 2],
    pub cards: [CardInst; MAX_CARDS],
    pub n_cards: u8,
    pub skip_opponent_turn: bool,
    pub last_attack: Option<AttackRef>,
    /// Indexed by player index (Twinleaf keys by player id).
    pub player_last_attack: [Option<(AttackRef, CardId)>; 2],
    /// `playerLastAttack[player].turn` (phase 4b): `state.turn` when it was set.
    pub player_last_attack_turn: [i32; 2],
    pub is_sudden_death: bool,
    pub bench_size_change_handled: bool,
    /// Players Twinleaf actually added (an invalid deck finishes the game
    /// before its AddPlayerAction adds the player).
    pub players_added: u8,
    pub ability_lock_order_counter: u16,
    /// Events batch 8: a counter bumped by every write to the game (the event routines, the physical moves, every
    /// interpreter op step and continuation, every action and answer): a derived fact computed at this version is still
    /// good (`derived.rs`; `PTCG_VERIFY_CACHE=1` checks a digest of the state against a missed bump).
    pub version: u32,
    /// Bumped where a card enters or leaves a zone list, a list is reordered, or the Active / Bench layout changes (the
    /// physical moves, `Game::lst_mut`, the Bench edits): what the in-place filter of the declaration sets and the
    /// lock-stamp gate read.
    pub layout_gen: u32,
    /// Per card id: the list the card was last put into ([`Loc`]), a hint every location read validates (the card must
    /// still be there) before trusting it, so a write that doesn't update it costs a scan, never a wrong answer
    /// (`PTCG_VERIFY_CACHE=1` checks every answer against the scan).
    pub at: [Loc; MAX_CARDS],
    /// What the Ability-lock stamps were last synced at (`passive::lock_sync`): the layout generation, the turn, the turn
    /// player and whether Stadium and Tool cards had no effect; the sync is skipped while they are the same.
    pub lock_synced: Option<(u32, i32, u8, bool)>,
    /// The lasting effects attacks left on Pokémon and players (events batch 8 C7; `engine::lasting`), in the order they
    /// were put.
    pub lasting: SVec<LastingEffect, 32>,
}

/// Where a card is (a [`State::at`] entry): a zone list of a player, a Pokémon spot's cards, or a Pokémon's Tools; `NONE`
/// for a card in a staging list or not known. One byte: the player in bit 7, the list in bits 0..6.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Loc(pub u8);

impl Loc {
    pub const NONE: Loc = Loc(0xFF);
    const DECK: u8 = 0;
    const HAND: u8 = 1;
    const DISCARD: u8 = 2;
    const LOST: u8 = 3;
    const STADIUM: u8 = 4;
    const SUPPORTER: u8 = 5;
    const PRIZE: u8 = 6;
    const SLOT: u8 = 12;
    const TOOL: u8 = 21;

    /// The location of a card put into the list `r` (`NONE` for a staging list).
    #[inline]
    pub fn of(r: ListRef) -> Loc {
        let (p, k) = match r {
            ListRef::Deck(p) => (p, Self::DECK),
            ListRef::Hand(p) => (p, Self::HAND),
            ListRef::Discard(p) => (p, Self::DISCARD),
            ListRef::LostZone(p) => (p, Self::LOST),
            ListRef::Stadium(p) => (p, Self::STADIUM),
            ListRef::Supporter(p) => (p, Self::SUPPORTER),
            ListRef::Prize(p, i) => (p, Self::PRIZE + i),
            ListRef::Slot(p, s) => (p, Self::SLOT + s),
            ListRef::SlotEnergies(..) | ListRef::Temp(_) => return Loc::NONE,
        };
        Loc(p << 7 | k)
    }

    /// A Tool attached to the Pokémon in spot `s` of player `p`.
    #[inline]
    pub fn tool(p: usize, s: SlotId) -> Loc {
        Loc((p as u8) << 7 | (Self::TOOL + s))
    }

    /// The list (not a Tool's place).
    #[inline]
    pub fn list(self) -> Option<ListRef> {
        if self == Loc::NONE {
            return None;
        }
        let p = self.0 >> 7;
        let k = self.0 & 0x7F;
        Some(match k {
            Self::DECK => ListRef::Deck(p),
            Self::HAND => ListRef::Hand(p),
            Self::DISCARD => ListRef::Discard(p),
            Self::LOST => ListRef::LostZone(p),
            Self::STADIUM => ListRef::Stadium(p),
            Self::SUPPORTER => ListRef::Supporter(p),
            k if k < Self::SLOT => ListRef::Prize(p, k - Self::PRIZE),
            k if k < Self::TOOL => ListRef::Slot(p, k - Self::SLOT),
            _ => return None,
        })
    }

    /// The spot whose Tools the card is among.
    #[inline]
    pub fn tool_slot(self) -> Option<(usize, SlotId)> {
        let k = self.0 & 0x7F;
        if self == Loc::NONE || k < Self::TOOL || k >= Self::TOOL + MAX_SLOTS as u8 {
            return None;
        }
        Some(((self.0 >> 7) as usize, k - Self::TOOL))
    }
}

impl State {
    pub fn new() -> State {
        State {
            phase: GamePhase::WaitingForPlayers,
            turn: 0,
            active_player: 0,
            winner: WINNER_NONE,
            end_reason: None,
            players: [Player::new(1), Player::new(2)],
            cards: [CardInst::default(); MAX_CARDS],
            n_cards: 0,
            skip_opponent_turn: false,
            last_attack: None,
            player_last_attack: [None, None],
            player_last_attack_turn: [0, 0],
            is_sudden_death: false,
            bench_size_change_handled: false,
            players_added: 2,
            ability_lock_order_counter: 0,
            version: 1,
            layout_gen: 0,
            at: [Loc::NONE; MAX_CARDS],
            lock_synced: None,
            lasting: SVec::new(),
        }
    }

    /// A write happened: facts derived at the previous version are stale (`derived.rs`).
    #[inline]
    pub fn bump(&mut self) {
        // 0 marks an empty memo entry (`derived.rs`).
        self.version = self.version.wrapping_add(1).max(1);
    }

    /// A card entered or left a zone list, a list was reordered, or the Active / Bench layout changed (also a write).
    #[inline]
    pub fn bump_layout(&mut self) {
        self.layout_gen = self.layout_gen.wrapping_add(1);
        self.version = self.version.wrapping_add(1).max(1);
    }

    /// Card `c` was put into the list `r` (a hint for [`State::locate`]).
    #[inline]
    pub fn put_at(&mut self, c: CardId, r: ListRef) {
        if (c as usize) < MAX_CARDS {
            self.at[c as usize] = Loc::of(r);
        }
    }

    /// Is spot `s` of player `p` the Active Spot or a Bench spot?
    #[inline]
    pub fn spot_in_play(&self, p: usize, s: SlotId) -> bool {
        let pl = &self.players[p];
        pl.active == s || pl.bench.contains(&s)
    }

    #[inline]
    pub fn cdef(&self, c: CardId) -> &'static CardDef {
        def(self.cards[c as usize].def)
    }
    #[inline]
    pub fn owner(&self, c: CardId) -> usize {
        self.cards[c as usize].owner as usize
    }
    pub fn slot(&self, p: usize, s: SlotId) -> &Slot {
        &self.players[p].slots[s as usize]
    }
    pub fn slot_mut(&mut self, p: usize, s: SlotId) -> &mut Slot {
        &mut self.players[p].slots[s as usize]
    }
    pub fn opp(p: usize) -> usize {
        1 - p
    }
    pub fn player_index_by_id(&self, id: u8) -> usize {
        if self.players[0].id == id {
            0
        } else {
            1
        }
    }

    // ---- PokemonCardList queries --------------------------------------

    /// `getPokemons()`.
    pub fn slot_pokemons(&self, p: usize, s: SlotId) -> SVec<CardId, 8> {
        let slot = self.slot(p, s);
        let mut out = SVec::new();
        for c in slot.cards.iter() {
            let f = crate::carddb::pokemon_flags(self.cards[c as usize].def);
            if (f & PF_POKEMON != 0 && !slot.tools.contains(c) && !slot.energies.contains(c)) || f & PF_FOSSIL != 0 {
                out.push(c);
            }
        }
        out
    }

    /// `getPokemonCard()`: the top Pokémon of the slot (the last Pokémon card of its list: scanned from the end).
    #[inline]
    pub fn slot_pokemon(&self, p: usize, s: SlotId) -> Option<CardId> {
        let slot = self.slot(p, s);
        for &c in slot.cards.as_slice().iter().rev() {
            let f = crate::carddb::pokemon_flags(self.cards[c as usize].def);
            if f & PF_FOSSIL != 0 || (f & PF_POKEMON != 0 && !slot.tools.contains(c) && !slot.energies.contains(c)) {
                return Some(c);
            }
        }
        None
    }

    /// Does any spot of either player hold a Special Condition (a plain read: a gate before the walks that look for
    /// an affected Pokémon)?
    #[inline]
    pub fn any_special_condition(&self) -> bool {
        self.players.iter().any(|pl| pl.slots.iter().any(|s| !s.special_conditions.is_empty()))
    }

    /// A fresh (empty) spot for player `p` (`Player::alloc_slot`), without the lasting effects a spot of that number had.
    pub fn alloc_slot(&mut self, p: usize) -> SlotId {
        let s = self.players[p].alloc_slot();
        self.lasting.retain(|l| l.on != LastingOn::pokemon(p, s));
        s
    }

    pub fn active_pokemon(&self, p: usize) -> Option<CardId> {
        self.slot_pokemon(p, self.players[p].active)
    }

    /// Where a card currently is (for `findCardList` style lookups): the first of the Active Spot, deck, discard pile,
    /// hand, Lost Zone, Stadium, Supporter, Bench spots and Prizes of player 0, then of player 1, that holds it (a Tool is
    /// in none). Answered from the hint [`State::at`] when the card is still there, else by the scan.
    #[inline]
    pub fn locate(&self, c: CardId) -> Option<ListRef> {
        let r = self.locate_hinted(c);
        if crate::game::verify_cache() {
            assert_eq!(r, self.locate_scan(c), "State::at answered differently from the scan for card {c}");
        }
        r
    }

    #[inline]
    fn locate_hinted(&self, c: CardId) -> Option<ListRef> {
        if let Some(r) = self.at.get(c as usize).and_then(|l| l.list()) {
            if self.holds(r, c) {
                return Some(r);
            }
        }
        self.locate_scan(c)
    }

    /// Does the list `r` hold `c`, as [`State::locate`] scans it (only the in-play spots and the Prizes in use)?
    #[inline]
    fn holds(&self, r: ListRef, c: CardId) -> bool {
        match r {
            ListRef::Slot(p, s) => self.spot_in_play(p as usize, s) && self.players[p as usize].slots[s as usize].cards.contains(c),
            ListRef::Prize(p, i) => i < self.players[p as usize].prize_count && self.players[p as usize].prizes[i as usize].contains(c),
            ListRef::SlotEnergies(..) | ListRef::Temp(_) => false,
            _ => self.list(r).contains(c),
        }
    }

    /// The spot whose Tools hold the Tool `c`, among the Pokémon in play (Active first, then the Bench, player 0 first).
    /// Answered from the hint [`State::at`]: a Tool location that still holds it, or a zone list that still holds the
    /// card (a card in a zone list is in no Tool list: `attach::put_attached` takes an attached Tool out of the spot's
    /// cards); else by the scan.
    #[inline]
    pub fn tool_spot(&self, c: CardId) -> Option<(usize, SlotId)> {
        let hint = self.at.get(c as usize).copied().unwrap_or(Loc::NONE);
        if let Some((p, s)) = hint.tool_slot() {
            let slot = &self.players[p].slots[s as usize];
            if self.spot_in_play(p, s) && !slot.cards.is_empty() && slot.tools.contains(c) {
                if crate::game::verify_cache() {
                    assert_eq!(Some((p, s)), self.tool_spot_scan(c), "State::at answered differently from the scan for Tool {c}");
                }
                return Some((p, s));
            }
        } else if let Some(r) = hint.list() {
            if self.holds(r, c) {
                if crate::game::verify_cache() {
                    assert_eq!(None, self.tool_spot_scan(c), "card {c} is in a zone list and in a Tool list");
                }
                return None;
            }
        }
        self.tool_spot_scan(c)
    }

    fn tool_spot_scan(&self, c: CardId) -> Option<(usize, SlotId)> {
        for p in 0..2 {
            let pl = &self.players[p];
            for &s in std::iter::once(&pl.active).chain(pl.bench.iter()) {
                let slot = &pl.slots[s as usize];
                if !slot.cards.is_empty() && slot.tools.contains(c) {
                    return Some((p, s));
                }
            }
        }
        None
    }

    /// [`State::locate`] the slow way: every list scanned.
    pub fn locate_scan(&self, c: CardId) -> Option<ListRef> {
        for p in 0..2 {
            let pl = &self.players[p];
            if pl.slots[pl.active as usize].cards.contains(c) {
                return Some(ListRef::Slot(p as u8, pl.active));
            }
            if pl.deck.contains(c) {
                return Some(ListRef::Deck(p as u8));
            }
            if pl.discard.contains(c) {
                return Some(ListRef::Discard(p as u8));
            }
            if pl.hand.contains(c) {
                return Some(ListRef::Hand(p as u8));
            }
            if pl.lostzone.contains(c) {
                return Some(ListRef::LostZone(p as u8));
            }
            if pl.stadium.contains(c) {
                return Some(ListRef::Stadium(p as u8));
            }
            if pl.supporter.contains(c) {
                return Some(ListRef::Supporter(p as u8));
            }
            for &b in pl.bench.iter() {
                if pl.slots[b as usize].cards.contains(c) {
                    return Some(ListRef::Slot(p as u8, b));
                }
            }
            for i in 0..pl.prize_count as usize {
                if pl.prizes[i].contains(c) {
                    return Some(ListRef::Prize(p as u8, i as u8));
                }
            }
        }
        None
    }

    /// `StateUtils.findPokemonSlot`: slot whose cards, energies or tools contain `c`.
    pub fn find_pokemon_slot(&self, c: CardId) -> Option<(usize, SlotId)> {
        for p in 0..2 {
            let pl = &self.players[p];
            for s in pl.all_slots().iter() {
                let slot = &pl.slots[*s as usize];
                if slot.cards.contains(c) || slot.energies.contains(c) || slot.tools.contains(c) {
                    return Some((p, *s));
                }
            }
        }
        None
    }

    pub fn stadium_card(&self) -> Option<CardId> {
        for p in 0..2 {
            if let Some(c) = self.players[p].stadium.get(0) {
                return Some(c);
            }
        }
        None
    }

    // ---- list addressing --------------------------------------------------

    pub fn list(&self, r: ListRef) -> &dyn CardList {
        match r {
            ListRef::Deck(p) => &self.players[p as usize].deck,
            ListRef::Hand(p) => &self.players[p as usize].hand,
            ListRef::Discard(p) => &self.players[p as usize].discard,
            ListRef::LostZone(p) => &self.players[p as usize].lostzone,
            ListRef::Stadium(p) => &self.players[p as usize].stadium,
            ListRef::Supporter(p) => &self.players[p as usize].supporter,
            ListRef::Prize(p, i) => &self.players[p as usize].prizes[i as usize],
            ListRef::Slot(p, s) => &self.players[p as usize].slots[s as usize].cards,
            ListRef::SlotEnergies(p, s) => &self.players[p as usize].slots[s as usize].energies,
            ListRef::Temp(_) => panic!("temp lists live in the store"),
        }
    }

    pub fn list_mut(&mut self, r: ListRef) -> &mut dyn CardList {
        self.bump_layout();
        match r {
            ListRef::Deck(p) => &mut self.players[p as usize].deck,
            ListRef::Hand(p) => &mut self.players[p as usize].hand,
            ListRef::Discard(p) => &mut self.players[p as usize].discard,
            ListRef::LostZone(p) => &mut self.players[p as usize].lostzone,
            ListRef::Stadium(p) => &mut self.players[p as usize].stadium,
            ListRef::Supporter(p) => &mut self.players[p as usize].supporter,
            ListRef::Prize(p, i) => &mut self.players[p as usize].prizes[i as usize],
            ListRef::Slot(p, s) => &mut self.players[p as usize].slots[s as usize].cards,
            ListRef::SlotEnergies(p, s) => &mut self.players[p as usize].slots[s as usize].energies,
            ListRef::Temp(_) => panic!("temp lists live in the store"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

/// Address of a card list (Twinleaf passes `CardList` objects around).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListRef {
    Deck(u8),
    Hand(u8),
    Discard(u8),
    LostZone(u8),
    Stadium(u8),
    Supporter(u8),
    Prize(u8, u8),
    /// A board slot's `cards` (the slot itself when a PokemonCardList is meant).
    Slot(u8, SlotId),
    SlotEnergies(u8, SlotId),
    /// Scratch list owned by the store (prompt temporaries).
    Temp(u8),
}

impl ListRef {
    pub fn owner(self) -> Option<usize> {
        match self {
            ListRef::Deck(p)
            | ListRef::Hand(p)
            | ListRef::Discard(p)
            | ListRef::LostZone(p)
            | ListRef::Stadium(p)
            | ListRef::Supporter(p)
            | ListRef::Prize(p, _)
            | ListRef::Slot(p, _)
            | ListRef::SlotEnergies(p, _) => Some(p as usize),
            ListRef::Temp(_) => None,
        }
    }
    pub fn is_slot(self) -> bool {
        matches!(self, ListRef::Slot(..))
    }
}
