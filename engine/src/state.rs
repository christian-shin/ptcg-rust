//! Game state: plain `Copy` data, no heap. Cloning is a memcpy.
//!
//! Mirrors Twinleaf's `State` / `Player` / `PokemonCardList` / `CardList`,
//! keeping only fields that rules or pool cards read or write. Pokémon slots
//! live in a per-player arena so a slot keeps its identity when it moves
//! between the Active Spot and the Bench (Twinleaf swaps the objects).

use crate::carddb::{def, CardDef, DefId};
use crate::list::*;
use crate::markers::*;
use crate::types::*;

pub const MAX_CARDS: usize = 120;
pub const MAX_SLOTS: usize = 9;
pub const MAX_BENCH: usize = 8;

/// Zone lists hold up to 120: Twinleaf can duplicate cards (energies of a
/// fully moved Pokémon slot are pushed twice), so zones may exceed 60.
pub type Deck = List<120>;
pub type SlotId = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MarkerItem {
    pub name: MarkerName,
    pub source: CardId,
    pub source_type: SourceType,
    pub target_scope: TargetScope,
}

/// `Marker`: ordered list of marker items.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Marker {
    pub items: SVec<MarkerItem, 12>,
}

impl Marker {
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
}

/// `PokemonCardList`: one board slot.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub cards: List<60>,
    pub energies: List<48>,
    pub tools: List<4>,
    pub damage: i32,
    pub hp: i32,
    pub hp_bonus: i32,
    /// Order matters (between-turns processing iterates it).
    pub special_conditions: SVec<u8, 5>,
    pub poison_damage: i32,
    pub burn_damage: i32,
    pub confusion_damage: i32,
    pub marker: Marker,
    pub pokemon_played_turn: i32,
    pub ability_lock_activation_order: i32,
    pub sleep_flips: i32,
    pub board_effect: SVec<u8, 6>,
    /// `attacksThisTurn` (absent until first written).
    pub attacks_this_turn: Option<i32>,
    pub healed_this_turn: bool,
    pub cannot_be_healed_next_turn: bool,
    pub cannot_attack_next_turn: bool,
    pub cannot_attack_next_turn_pending: bool,
    pub cannot_retreat_next_turn: bool,
    pub cannot_retreat_next_turn_pending: bool,
    /// `cannotUseAttacksNextTurn` / `...Pending`: attack names.
    pub cannot_use_attacks_next_turn: SVec<&'static str, 4>,
    pub cannot_use_attacks_next_turn_pending: SVec<&'static str, 4>,
    pub damage_reduction_next_turn: i32,
    /// `blockedAttackNameNextTurn`.
    pub blocked_attack_name_next_turn: Option<&'static str>,
    pub is_public: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            cards: List::new(),
            energies: List::new(),
            tools: List::new(),
            damage: 0,
            hp: 0,
            hp_bonus: 0,
            special_conditions: SVec::new(),
            poison_damage: 10,
            burn_damage: 20,
            confusion_damage: 30,
            marker: Marker::default(),
            pokemon_played_turn: 0,
            ability_lock_activation_order: 0,
            sleep_flips: 1,
            board_effect: SVec::new(),
            attacks_this_turn: None,
            healed_this_turn: false,
            cannot_be_healed_next_turn: false,
            cannot_attack_next_turn: false,
            cannot_attack_next_turn_pending: false,
            cannot_retreat_next_turn: false,
            cannot_retreat_next_turn_pending: false,
            cannot_use_attacks_next_turn: SVec::new(),
            cannot_use_attacks_next_turn_pending: SVec::new(),
            damage_reduction_next_turn: 0,
            blocked_attack_name_next_turn: None,
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
    /// Briar's `extraPrizes` instance field.
    pub extra_prizes: bool,
    /// `this.attacks[i].barrage` written at runtime (Festival Lead cards);
    /// bit i = attack i. Card-object state: never reset, not canonical.
    pub attack_barrage: u8,
    /// Attacks whose serialized object now differs from the printed card
    /// (canonical `cards[...].attacks`, with a `barrage` key).
    pub attack_barrage_shown: u8,
}

impl Default for CardInst {
    fn default() -> Self {
        CardInst { def: 0, owner: 0, moved_to_active_this_turn: false, damage_taken_last_turn: 0, extra_prizes: false, attack_barrage: 0, attack_barrage_shown: 0 }
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
    pub stadium: List<4>,
    pub supporter: List<8>,
    pub prizes: [List<4>; 6],
    pub prize_count: u8,
    pub slots: [Slot; MAX_SLOTS],
    pub slot_used: [bool; MAX_SLOTS],
    pub active: SlotId,
    pub bench: SVec<SlotId, MAX_BENCH>,

    pub supporter_turn: i32,
    pub retreated_turn: i32,
    pub energy_played_turn: i32,
    pub stadium_played_turn: i32,
    pub stadium_used_turn: i32,
    pub marker: Marker,
    pub used_vstar: bool,
    pub used_gx: bool,
    pub prizes_taken: i32,
    pub prizes_taken_this_turn: i32,
    pub prizes_taken_last_turn: i32,
    pub moved_to_active_this_turn: SVec<CardId, 16>,
    pub moved_from_active_to_bench_this_turn: SVec<CardId, 16>,
    pub pokemon_knocked_out_during_opponents_last_turn: bool,
    pub pokemon_knocked_out_by_attack_during_opponents_last_turn: bool,
    pub pokemon_knocked_out_last_turn_entries: SVec<DefId, 8>,
    pub can_evolve: bool,
    pub ancient_pokemon_attacked_last_turn: bool,
    pub cannot_play_item_cards: bool,
    pub cannot_play_supporter_cards: bool,
    pub cannot_play_stadium_cards: bool,
    pub cannot_play_tool_cards: bool,
    pub cannot_play_special_energy_cards: bool,
    pub cannot_play_energy_cards: bool,
    pub cannot_play_pokemon_cards: bool,
    pub cannot_play_pokemon_with_abilities: bool,
    pub cannot_evolve_pokemon_cards: bool,
    pub play_locks_turns_remaining: i32,
    pub used_dragons_wish: bool,
    pub unlimited_energy_attach_turns_remaining: i32,
    pub cannot_draw_at_start_of_turn: bool,
    pub cannot_attack_turns_remaining: i32,
    pub stadium_and_tool_have_no_effect_turns_remaining: i32,
    pub coin_flip_cancel_trainer_play_turns_remaining: i32,
    /// `usedTableTurner` (Fezandipiti ex; absent until first written).
    pub used_table_turner: bool,
    /// Pecharunt ex's `chainsOfControlUsed` (absent until first written).
    pub chains_of_control_used: bool,
    /// `pecharuntexIsInPlay` (set by Pecharunt ex, never cleared).
    pub pecharuntex_is_in_play: bool,
    /// Mega Kangaskhan ex's `usedRunErrand` (absent until set).
    pub used_run_errand: bool,
    /// `rocketSupporter` (Team Rocket's Petrel; cleared at its owner's end of turn).
    pub rocket_supporter: bool,
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
            slots: [Slot::default(); MAX_SLOTS],
            slot_used: [false; MAX_SLOTS],
            active: 0,
            bench: SVec::new(),
            supporter_turn: 0,
            retreated_turn: 0,
            energy_played_turn: 0,
            stadium_played_turn: 0,
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
            can_evolve: false,
            ancient_pokemon_attacked_last_turn: false,
            cannot_play_item_cards: false,
            cannot_play_supporter_cards: false,
            cannot_play_stadium_cards: false,
            cannot_play_tool_cards: false,
            cannot_play_special_energy_cards: false,
            cannot_play_energy_cards: false,
            cannot_play_pokemon_cards: false,
            cannot_play_pokemon_with_abilities: false,
            cannot_evolve_pokemon_cards: false,
            play_locks_turns_remaining: 0,
            used_dragons_wish: false,
            unlimited_energy_attach_turns_remaining: 0,
            cannot_draw_at_start_of_turn: false,
            cannot_attack_turns_remaining: 0,
            stadium_and_tool_have_no_effect_turns_remaining: 0,
            coin_flip_cancel_trainer_play_turns_remaining: 0,
            used_table_turner: false,
            chains_of_control_used: false,
            pecharuntex_is_in_play: false,
            used_run_errand: false,
            rocket_supporter: false,
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

/// Rules toggles (Twinleaf `Rules` defaults).
#[derive(Clone, Copy, Debug)]
pub struct Rules {
    pub first_turn_draw_card: bool,
    pub first_turn_use_supporter: bool,
    pub attack_first_turn: bool,
    pub supporter_cleanup_at_end_turn: bool,
    pub unlimited_energy_attachments: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            first_turn_draw_card: true,
            first_turn_use_supporter: true,
            attack_first_turn: false,
            supporter_cleanup_at_end_turn: false,
            unlimited_energy_attachments: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub phase: GamePhase,
    pub turn: i32,
    pub active_player: u8,
    pub winner: Winner,
    pub players: [Player; 2],
    pub cards: [CardInst; MAX_CARDS],
    pub n_cards: u8,
    pub rules: Rules,
    pub skip_opponent_turn: bool,
    pub last_attack: Option<AttackRef>,
    /// Indexed by player index (Twinleaf keys by player id).
    pub player_last_attack: [Option<(AttackRef, CardId)>; 2],
    pub is_sudden_death: bool,
    pub bench_size_change_handled: bool,
    /// Players Twinleaf actually added (an invalid deck finishes the game
    /// before its AddPlayerAction adds the player).
    pub players_added: u8,
    pub ability_lock_order_counter: i32,
}

impl State {
    pub fn new() -> State {
        State {
            phase: GamePhase::WaitingForPlayers,
            turn: 0,
            active_player: 0,
            winner: WINNER_NONE,
            players: [Player::new(1), Player::new(2)],
            cards: [CardInst::default(); MAX_CARDS],
            n_cards: 0,
            rules: Rules::default(),
            skip_opponent_turn: false,
            last_attack: None,
            player_last_attack: [None, None],
            is_sudden_death: false,
            bench_size_change_handled: false,
            players_added: 2,
            ability_lock_order_counter: 0,
        }
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
            let d = self.cdef(c);
            if (d.is_pokemon() && !slot.tools.contains(c) && !slot.energies.contains(c)) || d.fossil_doll {
                out.push(c);
            }
        }
        out
    }

    /// `getPokemonCard()`: the top Pokémon of the slot.
    pub fn slot_pokemon(&self, p: usize, s: SlotId) -> Option<CardId> {
        let slot = self.slot(p, s);
        let mut top = None;
        for c in slot.cards.iter() {
            let d = self.cdef(c);
            if (d.is_pokemon() && !slot.tools.contains(c) && !slot.energies.contains(c)) || d.fossil_doll {
                top = Some(c);
            }
        }
        top
    }

    pub fn active_pokemon(&self, p: usize) -> Option<CardId> {
        self.slot_pokemon(p, self.players[p].active)
    }

    /// Where a card currently is (for `findCardList` style lookups).
    pub fn locate(&self, c: CardId) -> Option<ListRef> {
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
