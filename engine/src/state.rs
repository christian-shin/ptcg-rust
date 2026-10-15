//! Game state: plain `Copy` data, no heap. Cloning is a memcpy.
//!
//! The game, its players, their Pokémon slots and card lists, keeping only fields that rules or pool cards read or
//! write. Pokémon slots live in a per-player arena so a slot keeps its identity when it moves between the Active Spot
//! and the Bench.

use crate::carddb::{def, CardDef, DefId, PF_FOSSIL, PF_POKEMON};
use crate::list::*;
use crate::types::*;

/// `State::first_player` before the first player is decided.
pub const NO_PLAYER: u8 = u8::MAX;
pub const MAX_CARDS: usize = 120;
pub const MAX_SLOTS: usize = 9;
/// Pokémon references from both players at once (damaged lists, multi-Pokémon answers).
pub const MAX_SLOT_REFS: usize = 2 * MAX_SLOTS;
pub const MAX_BENCH: usize = 8;

/// Zone lists hold up to 120 cards, twice a deck's 60: headroom no legal game uses (a card is in one zone only).
pub type Deck = ZoneList<120>;
pub type SlotId = u8;

/// One board slot: the Pokémon, its attachments, damage and conditions.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    /// The Pokemon stack and the cards attached to it, in order. Private to this module: every write goes through
    /// [`SlotEdit`] (or [`ListMut`]), whose drop recomputes `top`, so `top` can't go stale.
    cards: ZoneList<60>,
    /// The Energy cards attached (also in `cards`, bar a Pokemon acting as Energy).
    energies: ZoneList<60>,
    tools: ZoneList<4>,
    /// The top Pokemon of the spot (`NO_CARD` for none): the last card of `cards` that is a Pokemon card not attached
    /// as a Tool or Energy, a Fossil counting as one. Kept by [`SlotEdit`]'s drop; `PTCG_VERIFY_CACHE=1` checks every
    /// read against [`Slot::top_scan`].
    top: CardId,
    pub damage: i32,
    /// Order matters (between-turns processing iterates it).
    pub special_conditions: SVec<u8, 5>,
    pub poison_damage: i32,
    pub burn_damage: i32,
    pub confusion_damage: i32,
    /// The turn the Pokémon in this spot came into play: put into play, evolved or devolved (APR A-05,
    /// C-13; id2229). Slot-bound (events design 4.5). No permission rewrites it: permissions lift limits
    /// (`Modifier::Permit`) instead. The one record of the fact: evolution timing, "played this turn"
    /// (`SlotPred::PlayedThisTurn`), the observation and the canonical JSON read it. The
    /// Pokémon set up before the first turn count as entering on their owner's first turn (1 or 2,
    /// `setup::finish`).
    pub entered_turn: i32,
    pub sleep_flips: i32,
    /// The attacks this Pokémon used this turn (absent until first written).
    pub attacks_this_turn: Option<i32>,
    pub healed_this_turn: bool,
    /// An effect Knocked this Pokémon Out ("is Knocked Out", "Knock Out ..."; `engine::knockout::by_effect`): it is
    /// Knocked Out at the next state check with every other Knock Out (user decision D1), by this cause.
    pub ko_pending: Option<crate::cause::Cause>,
    pub is_public: bool,
}

/// The occupied spots of a player, the Active Spot first and then the Bench in order ([`Player::in_play`]).
#[derive(Clone, Copy)]
pub struct InPlay {
    occupied: u16,
    active: SlotId,
    bench: SVec<SlotId, MAX_BENCH>,
    next: u8,
}

impl InPlay {
    /// No spot of the player holds a card.
    pub fn is_empty(mut self) -> bool {
        self.next().is_none()
    }
}

impl Iterator for InPlay {
    type Item = SlotId;
    #[inline]
    fn next(&mut self) -> Option<SlotId> {
        loop {
            let s = if self.next == 0 { self.active } else { *self.bench.as_slice().get(self.next as usize - 1)? };
            self.next += 1;
            if self.occupied >> s & 1 != 0 {
                return Some(s);
            }
        }
    }
}

impl Slot {
    /// The cards of the spot: the Pokemon stack and what is attached to it.
    #[inline]
    pub fn cards(&self) -> &ZoneList<60> {
        &self.cards
    }
    #[inline]
    pub fn energies(&self) -> &ZoneList<60> {
        &self.energies
    }
    #[inline]
    pub fn tools(&self) -> &ZoneList<4> {
        &self.tools
    }
    /// The top Pokemon of the spot (the kept field).
    #[inline]
    pub fn top(&self) -> Option<CardId> {
        if self.top == NO_CARD {
            None
        } else {
            Some(self.top)
        }
    }

    /// The top Pokemon by the scan: the last card of the list that is a Pokemon card not attached as a Tool or Energy
    /// (a Fossil counts as one). The definition `top` is kept equal to.
    pub fn top_scan(&self, insts: &[CardInst; MAX_CARDS]) -> Option<CardId> {
        for &c in self.cards.as_slice().iter().rev() {
            let f = crate::carddb::pokemon_flags(insts[c as usize].def);
            if f & PF_FOSSIL != 0 || (f & PF_POKEMON != 0 && !self.tools.contains(c) && !self.energies.contains(c)) {
                return Some(c);
            }
        }
        None
    }
}

/// The one write path to a spot's card lists: it lends the lists out mutably and, on drop, recomputes the spot's top
/// Pokemon and whether it holds a card (`Player::occupied`) from them. A list is private to this module, so no other write to it can exist.
pub struct SlotEdit<'a> {
    slot: &'a mut Slot,
    /// The player's `occupied` mask and this spot's bit in it.
    occupied: &'a mut u16,
    bit: u16,
    insts: &'a mut [CardInst; MAX_CARDS],
    /// The game's arrival counter (`State::arrival`).
    arrival: &'a mut u16,
    /// The spot's lists as they were when the guard was taken: a card on a list now and on none then has arrived.
    before: (ZoneList<60>, ZoneList<60>, ZoneList<4>),
}

impl SlotEdit<'_> {
    #[inline]
    pub fn cards(&mut self) -> &mut ZoneList<60> {
        &mut self.slot.cards
    }
    #[inline]
    pub fn energies(&mut self) -> &mut ZoneList<60> {
        &mut self.slot.energies
    }
    #[inline]
    pub fn tools(&mut self) -> &mut ZoneList<4> {
        &mut self.slot.tools
    }
    /// The spot after "no Pokemon is on it any more" (`reset_empty_slot`): its cards and Energy stay, its Tools and every
    /// other field return to a fresh spot's, bar whether it is public.
    pub fn reset_keeping_cards(&mut self) {
        let keep = Slot { cards: self.slot.cards, energies: self.slot.energies, is_public: self.slot.is_public, ..Slot::default() };
        *self.slot = keep;
    }
}

impl SlotEdit<'_> {
    /// Cards that are on the spot's lists now and were on none of them when the guard was taken get the next arrival
    /// stamp, in the order of `cards`, then `energies`, then `tools` (a card on two lists is stamped once). Saturates at
    /// `u16::MAX` (later arrivals tie and keep list order); a game doesn't come near it.
    fn stamp_arrivals(&mut self) {
        let (c, e, t) = (self.slot.cards.as_slice(), self.slot.energies.as_slice(), self.slot.tools.as_slice());
        let (bc, be, bt) = (self.before.0.as_slice(), self.before.1.as_slice(), self.before.2.as_slice());
        if c == bc && e == be && t == bt {
            return;
        }
        let was = |x: CardId| bc.contains(&x) || be.contains(&x) || bt.contains(&x);
        for (i, &x) in c.iter().chain(e).chain(t).enumerate() {
            let dup = (i >= c.len() && c.contains(&x)) || (i >= c.len() + e.len() && e.contains(&x));
            if !dup && !was(x) {
                *self.arrival = self.arrival.saturating_add(1);
                self.insts[x as usize].arrived = *self.arrival;
            }
        }
    }
}

impl Drop for SlotEdit<'_> {
    #[inline]
    fn drop(&mut self) {
        self.stamp_arrivals();
        self.slot.top = self.slot.top_scan(self.insts).unwrap_or(NO_CARD);
        if self.slot.cards.is_empty() {
            *self.occupied &= !self.bit;
        } else {
            *self.occupied |= self.bit;
        }
    }
}

/// A mutable card list of any kind: a plain list, or one of a spot's two lists behind its [`SlotEdit`].
pub enum ListMut<'a> {
    Plain(&'a mut dyn CardList),
    SlotCards(SlotEdit<'a>),
    SlotEnergies(SlotEdit<'a>),
}

impl CardList for ListMut<'_> {
    #[inline]
    fn as_slice(&self) -> &[CardId] {
        match self {
            ListMut::Plain(l) => l.as_slice(),
            ListMut::SlotCards(e) => e.slot.cards.as_slice(),
            ListMut::SlotEnergies(e) => e.slot.energies.as_slice(),
        }
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [CardId] {
        match self {
            ListMut::Plain(l) => l.as_mut_slice(),
            ListMut::SlotCards(e) => e.slot.cards.as_mut_slice(),
            ListMut::SlotEnergies(e) => e.slot.energies.as_mut_slice(),
        }
    }
    #[inline]
    fn push(&mut self, c: CardId) {
        match self {
            ListMut::Plain(l) => l.push(c),
            ListMut::SlotCards(e) => e.slot.cards.push(c),
            ListMut::SlotEnergies(e) => e.slot.energies.push(c),
        }
    }
    fn insert(&mut self, i: usize, c: CardId) {
        match self {
            ListMut::Plain(l) => l.insert(i, c),
            ListMut::SlotCards(e) => e.slot.cards.insert(i, c),
            ListMut::SlotEnergies(e) => e.slot.energies.insert(i, c),
        }
    }
    fn remove_at(&mut self, i: usize) -> CardId {
        match self {
            ListMut::Plain(l) => l.remove_at(i),
            ListMut::SlotCards(e) => e.slot.cards.remove_at(i),
            ListMut::SlotEnergies(e) => e.slot.energies.remove_at(i),
        }
    }
    #[inline]
    fn clear(&mut self) {
        match self {
            ListMut::Plain(l) => l.clear(),
            ListMut::SlotCards(e) => e.slot.cards.clear(),
            ListMut::SlotEnergies(e) => e.slot.energies.clear(),
        }
    }
    fn set_from(&mut self, cards: &[CardId]) {
        match self {
            ListMut::Plain(l) => l.set_from(cards),
            ListMut::SlotCards(e) => e.slot.cards.set_from(cards),
            ListMut::SlotEnergies(e) => e.slot.energies.set_from(cards),
        }
    }
    fn capacity(&self) -> usize {
        match self {
            ListMut::Plain(l) => l.capacity(),
            ListMut::SlotCards(e) => e.slot.cards.capacity(),
            ListMut::SlotEnergies(e) => e.slot.energies.capacity(),
        }
    }
    fn as_permutable_slice(&mut self) -> &mut [CardId] {
        match self {
            ListMut::Plain(l) => l.as_permutable_slice(),
            ListMut::SlotCards(e) => e.slot.cards.as_permutable_slice(),
            ListMut::SlotEnergies(e) => e.slot.energies.as_permutable_slice(),
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            cards: List::new(),
            energies: List::new(),
            tools: List::new(),
            top: NO_CARD,
            damage: 0,
            special_conditions: SVec::new(),
            poison_damage: 10,
            burn_damage: 20,
            confusion_damage: 30,
            entered_turn: 0,
            sleep_flips: 1,
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
    /// The arrival stamp: the number (from `State::arrival`, 1 and up) a Pokémon spot's write guard ([`SlotEdit`]) gave the
    /// card when it last entered a spot's lists (put on the Bench, evolved onto, attached, moved to another Pokémon); 0
    /// until it first does. Meaningful while the card is in a spot; stale after it leaves (re-entering stamps it again).
    /// The Energy and Tool prompts list attached cards in this order (`interface.rs`).
    pub arrived: u16,
}

impl Default for CardInst {
    fn default() -> Self {
        CardInst { def: 0, owner: 0, moved_to_active_this_turn: false, damage_taken_last_turn: 0, lock_stamp: 0, staged_from: None, staged_in: None, arrived: 0 }
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

/// When a record (a usage record, a lasting declared effect or a slot record) ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Until {
    /// "This turn", player `p`'s: it ends when their turn ends, in the EndTurn event, before the end-of-turn effects the
    /// player orders (an expiry is not an effect: APR E-21).
    ThisTurn(u8),
    /// Until something else removes it (a card's text, the Pokémon's effects ending, the card leaving play, the game's end).
    Removed,
}

/// What a usage record is about: "you can't use more than 1 X Ability each turn" is per printed Ability name; a card's
/// own declaration (a Tool's "once during your turn", a Special Energy's "once per game") is per card definition and
/// declaration.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum UsageKey {
    /// The printed Ability name (`PowerDef::name_id`, generated into `gen::cards::ABILITY_NAMES`).
    Ability(u16),
    /// The `index`th declaration of the card `def` (a passive: its position in `CardSpec::passives`; a power:
    /// `0x80 | its index`; `0xFF`: the card's own record).
    Declared { def: crate::carddb::DefId, index: u8 },
}

/// Whose record it is: a player's (the limits that are the player's: the same Ability of every copy, "once per game", a Tool's
/// once) or a card's (a Pokémon's own use, id317: it ends with the card's entering or leaving play, or being replaced; a
/// Pokémon Knocked Out: "doesn't stack").
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum UsageBy {
    Player(u8),
    Card(CardId),
}

/// A usage record: the rules concept behind "once during your turn", "you can't use more than 1 X Ability each turn",
/// "once per game" and "the effect doesn't stack" (events batch 9 C6; `engine::usage`). `State::usage` holds them all.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Usage {
    pub key: UsageKey,
    pub by: UsageBy,
    pub until: Until,
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
    /// Pokémon" (the source's attack `attack`, by player `attacker`; e.g. Little Grudge).
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
    /// An effect the source card's text declares and puts for a time (`CardSpec::effects`, `Op::Declare`): "during this turn,
    /// attacks used by your Pokémon do 30 more damage" (a Trainer's effect on its player, APR C-19; an Ability's on its Pokémon,
    /// C-15). The declaration number `index` of the source card, until `until`. Read where its modifier is read for a card in
    /// place (`passive::declared_effects`); the card that put it needn't be in play any more.
    Declared { index: u8, until: Until },
    /// A record of the source card on the Pokémon ("that Pokémon": Boomerang Energy's, Heavy Baton's; `Op::Record`): it stays
    /// when the Pokémon moves to the Bench and ends when it evolves, is devolved or leaves play, or when a card's text forgets it
    /// (`Op::Forget`).
    Recorded,
}

impl LastingWhat {
    /// The turns of a lock that lasts until the Pokémon's effects end ("until it leaves the Active Spot").
    pub const UNTIL_REMOVED: i8 = i8::MAX;

    /// Nothing in force nor pending: the entry is removed.
    pub fn is_spent(&self) -> bool {
        match *self {
            LastingWhat::Lock { .. } | LastingWhat::PlayerLock { .. } | LastingWhat::Prevent { .. } | LastingWhat::Retaliate { .. } | LastingWhat::NextTurnBonus { .. } | LastingWhat::DiscardAttackerEnergy { .. } | LastingWhat::Spawn { .. } | LastingWhat::Declared { .. } | LastingWhat::Recorded => false,
            LastingWhat::TakesLessDamage(n) | LastingWhat::DealsLessDamage(n) => n == 0,
            LastingWhat::TakesMoreDamage { amount, attacker, pending, rearm } => amount == 0 && attacker.is_none() && !pending && !rearm,
            LastingWhat::AttackCostIncrease { now, pending, attacker } | LastingWhat::RetreatCostIncrease { now, pending, attacker } => now == 0 && pending == 0 && attacker.is_none(),
            LastingWhat::NoWeakness { armed, pending } => !armed && !pending,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub deck: Deck,
    pub hand: Deck,
    pub discard: Deck,
    pub lostzone: Deck,
    pub stadium: ZoneList<4>,
    pub supporter: ZoneList<8>,
    pub prizes: [ZoneList<4>; 6],
    pub prize_count: u8,
    /// The Prize card is public (Cresselia SFA turns one public);
    /// all Prize cards start secret.
    pub prize_public: [bool; 6],
    /// The Prize card is face up.
    pub prize_face_up: [bool; 6],
    pub slots: [Slot; MAX_SLOTS],
    /// Bit `s` is set when spot `s` holds a card. Kept by [`SlotEdit`]'s drop and `alloc_slot`; read only through
    /// [`Player::in_play`] (`PTCG_VERIFY_CACHE=1` checks it against the scan of the lists).
    occupied: u16,
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
}

impl Player {
    /// A player with 6 face-down prize lists, 5 empty bench slots and no cards.
    pub fn new() -> Player {
        let mut p = Player {
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
            occupied: 0,
            slot_used: [false; MAX_SLOTS],
            active: 0,
            bench: SVec::new(),
            played_this_turn: SVec::new(),
            retreated_turn: 0,
            energy_played_turn: 0,
            stadium_used_turn: 0,
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

    pub fn bench_slot(&self, i: usize) -> &Slot {
        &self.slots[self.bench.as_slice()[i] as usize]
    }
    /// The occupied slots: the Active Spot, then the Bench in order (read from the kept `occupied` mask; a `Copy`
    /// iterator that borrows nothing, so the state can be written while it is walked).
    #[inline]
    pub fn in_play(&self) -> InPlay {
        let r = InPlay { occupied: self.occupied, active: self.active, bench: self.bench, next: 0 };
        if crate::game::verify_cache() {
            let mut scan: SVec<SlotId, MAX_SLOTS> = SVec::new();
            for s in std::iter::once(self.active).chain(self.bench.iter().copied()) {
                if !self.slots[s as usize].cards().is_empty() {
                    scan.push(s);
                }
            }
            assert!(r.eq(scan.iter().copied()), "Player::occupied answered differently from the scan of the spots");
        }
        r
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
                self.occupied &= !(1 << i);
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
    /// clone (see `copy_attack.rs`): bits 4..6 hold the
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
    /// The player who takes the first turn of the game, `NO_PLAYER` until it is decided (setup, after the coin flip and
    /// the choice; the Tiebreaker game sets it again).
    pub first_player: u8,
    pub winner: Winner,
    /// Why the game ended (`engine::knockout::game_end`).
    pub end_reason: Option<crate::engine::knockout::EndReason>,
    pub players: [Player; 2],
    pub cards: [CardInst; MAX_CARDS],
    pub n_cards: u8,
    pub skip_opponent_turn: bool,
    pub last_attack: Option<AttackRef>,
    /// Indexed by player index.
    pub player_last_attack: [Option<(AttackRef, CardId)>; 2],
    /// The turn on which each player's last attack was set.
    pub player_last_attack_turn: [i32; 2],
    pub is_sudden_death: bool,
    pub bench_size_change_handled: bool,
    /// The players added to the game (an invalid deck finishes the game before its player is added).
    pub players_added: u8,
    pub ability_lock_order_counter: u16,
    /// The last arrival stamp given (`CardInst::arrived`); 0 before any card entered a spot.
    pub arrival: u16,
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
    /// What the Ability-lock stamps were last synced at (`passive::lock_sync`): the layout generation, the turn and the turn
    /// player; the sync is skipped while they are the same.
    pub lock_synced: Option<(u32, i32, u8)>,
    /// The lasting effects attacks left on Pokémon and players (events batch 8 C7; `engine::lasting`), in the order they
    /// were put.
    pub lasting: SVec<LastingEffect, 32>,
    /// The usage records (events batch 9 C6; `engine::usage`), in the order they were made.
    pub usage: SVec<Usage, 32>,
    /// The cards that may hold an Ability usage record (`UsageBy::Card`, `UsageKey::Ability`), one bit per card id: set when
    /// the record is made, cleared when the card's records are reset (`usage::reset_card`); a bit may stay set after the
    /// record expired with its turn, never the other way (a cache of "scan the records for this card", `engine::usage`).
    pub usage_held: [u64; 2],
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
            first_player: NO_PLAYER,
            winner: WINNER_NONE,
            end_reason: None,
            players: [Player::new(), Player::new()],
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
            arrival: 0,
            version: 1,
            layout_gen: 0,
            at: [Loc::NONE; MAX_CARDS],
            lock_synced: None,
            lasting: SVec::new(),
            usage: SVec::new(),
            usage_held: [0; 2],
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

    // ---- PokemonCardList queries --------------------------------------

    /// The Pokémon cards of a slot (the Pokémon on it, fossils and dolls that count as one included).
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

    /// The top Pokémon of the slot: the kept field (`Slot::top`), asserted against the scan under `PTCG_VERIFY_CACHE=1`.
    #[inline]
    pub fn slot_pokemon(&self, p: usize, s: SlotId) -> Option<CardId> {
        let slot = self.slot(p, s);
        let r = slot.top();
        if crate::game::verify_cache() {
            assert_eq!(r, slot.top_scan(&self.cards), "Slot::top answered differently from the scan for spot {s} of player {p}");
        }
        r
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

    /// Where a card currently is (for lookups): the first of the Active Spot, deck, discard pile,
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

    /// The slot whose cards, energies or tools contain `c`.
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

    pub fn list_mut(&mut self, r: ListRef) -> ListMut<'_> {
        self.bump_layout();
        match r {
            ListRef::Slot(p, s) => ListMut::SlotCards(self.slot_edit(p as usize, s)),
            ListRef::SlotEnergies(p, s) => ListMut::SlotEnergies(self.slot_edit(p as usize, s)),
            _ => ListMut::Plain(self.plain_list_mut(r)),
        }
    }

    /// The one write path to a spot's card lists (see [`SlotEdit`]).
    #[inline]
    pub fn slot_edit(&mut self, p: usize, s: SlotId) -> SlotEdit<'_> {
        let pl = &mut self.players[p];
        let slot = &mut pl.slots[s as usize];
        let before = (slot.cards, slot.energies, slot.tools);
        SlotEdit { slot, occupied: &mut pl.occupied, bit: 1 << s, insts: &mut self.cards, arrival: &mut self.arrival, before }
    }

    fn plain_list_mut(&mut self, r: ListRef) -> &mut dyn CardList {
        match r {
            ListRef::Deck(p) => &mut self.players[p as usize].deck,
            ListRef::Hand(p) => &mut self.players[p as usize].hand,
            ListRef::Discard(p) => &mut self.players[p as usize].discard,
            ListRef::LostZone(p) => &mut self.players[p as usize].lostzone,
            ListRef::Stadium(p) => &mut self.players[p as usize].stadium,
            ListRef::Supporter(p) => &mut self.players[p as usize].supporter,
            ListRef::Prize(p, i) => &mut self.players[p as usize].prizes[i as usize],
            ListRef::Slot(..) | ListRef::SlotEnergies(..) => unreachable!("a spot's lists are written through SlotEdit"),
            ListRef::Temp(_) => panic!("temp lists live in the store"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

/// Address of a card list.
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
}

#[cfg(test)]
mod tests {
    //! `Slot::top` against the scan, through every kind of write to a spot's lists.
    use super::*;
    use crate::game::Game;
    use serde_json::json;

    const DURA: &str = "Duraludon PRE 69";
    const MEWTWO: &str = "Team Rocket's Mewtwo ex DRI 81";
    const ENERGY: &str = "Metal Energy MEE 8";
    const CHARM: &str = "Sacred Charm PFL 93";

    fn game() -> Game {
        game_with(false)
    }

    /// The scenario board, loaded while the setup prompts are pending, or (`to_the_turn`) after the setup is played out
    /// with the first valid answers, at the first turn decision.
    fn game_with(to_the_turn: bool) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for (n, k) in [(DURA, 4), (MEWTWO, 3), (CHARM, 4)] {
            names.extend(std::iter::repeat(n).take(k));
        }
        while names.len() < 60 {
            names.push(ENERGY);
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        while to_the_turn {
            let sel = g.select_with(false).unwrap().unwrap();
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
        let sc = json!({"me": {"reset": true, "active": DURA, "active_energy": [ENERGY], "active_tool": CHARM, "bench": [{"card": MEWTWO}, {"card": DURA}]}, "opp": {"reset": true, "active": DURA}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// Every spot of both players: the kept top equals the scan.
    fn assert_tops(g: &Game) {
        for p in 0..2 {
            for s in 0..MAX_SLOTS {
                let sl = &g.st.players[p].slots[s];
                assert_eq!(sl.top(), sl.top_scan(&g.st.cards), "spot {s} of player {p}");
            }
        }
    }

    /// A copy of `name` of player `p` in its deck, hand or Prizes, and where it is.
    fn find(g: &Game, p: usize, name: &str) -> (CardId, ListRef) {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        [ListRef::Deck(p as u8), ListRef::Hand(p as u8)]
            .into_iter()
            .chain((0..6).map(|i| ListRef::Prize(p as u8, i)))
            .find_map(|l| g.lst(l).iter().copied().find(|c| g.st.cards[*c as usize].def == def).map(|c| (c, l)))
            .unwrap()
    }

    #[test]
    fn top_is_right_after_scenario_loading() {
        let g = game();
        assert_tops(&g);
        let a = g.st.players[0].active;
        let top = g.st.slot_pokemon(0, a).unwrap();
        assert_eq!(g.st.cards[top as usize].def, crate::carddb::def_by_full_name(DURA).unwrap());
        assert!(g.st.slot(0, a).tools().len() == 1 && g.st.slot(0, a).energies().len() == 1);
    }

    #[test]
    fn top_follows_attach_swap_devolve_and_knock_out() {
        let mut g = game();
        let a = g.st.players[0].active;
        let first = g.st.slot_pokemon(0, a).unwrap();
        // Evolve: a Pokemon card put on top of the stack.
        let (mew, mew_from) = find(&g, 0, MEWTWO);
        g.move_card_to(mew_from, mew, ListRef::Slot(0, a));
        assert_eq!(g.st.slot_pokemon(0, a), Some(mew));
        assert_tops(&g);
        // Attach an Energy: the top stays.
        let (en, en_from) = find(&g, 0, ENERGY);
        g.move_card_to(en_from, en, ListRef::Slot(0, a));
        assert_eq!(g.st.slot_pokemon(0, a), Some(mew));
        assert_tops(&g);
        // Devolve: the top goes to the hand and the card under it is the top.
        g.move_card_to(ListRef::Slot(0, a), mew, ListRef::Hand(0));
        assert_eq!(g.st.slot_pokemon(0, a), Some(first));
        assert_tops(&g);
        // Swap: the Pokemon leaves and another takes its place.
        let (dura, dura_from) = find(&g, 0, DURA);
        g.move_card_to(ListRef::Slot(0, a), first, ListRef::Hand(0));
        g.move_card_to(dura_from, dura, ListRef::Slot(0, a));
        assert_eq!(g.st.slot_pokemon(0, a), Some(dura));
        assert_tops(&g);
        // The Tool and Energy are never the top.
        let t = g.st.slot(0, a).tools().as_slice()[0];
        assert_ne!(g.st.slot_pokemon(0, a), Some(t));
        // Knock Out of a Bench spot: every card of the spot leaves, the spot has no Pokemon.
        let b = g.st.players[0].bench.as_slice()[0];
        crate::engine::cards_zone::relocate_spot(&mut g, crate::effects::SlotRef::new(0, b), ListRef::Discard(0));
        assert_eq!(g.st.slot_pokemon(0, b), None);
        assert_tops(&g);
        // Bench shrink: a spot taken off the Bench keeps a consistent top.
        g.st.players[0].bench.clear();
        assert_tops(&g);
    }

    /// `Player::occupied` against the scan of the lists, and `in_play` in the order Active then Bench.
    fn assert_occupied(g: &Game) {
        for pl in g.st.players.iter() {
            for s in 0..MAX_SLOTS {
                if pl.slot_used[s] {
                    assert_eq!(pl.occupied >> s & 1 != 0, !pl.slots[s].cards().is_empty(), "spot {s}");
                }
            }
            let want: Vec<SlotId> = std::iter::once(pl.active).chain(pl.bench.iter().copied()).filter(|&s| !pl.slots[s as usize].cards().is_empty()).collect();
            assert_eq!(pl.in_play().collect::<Vec<_>>(), want);
        }
    }

    #[test]
    fn occupied_follows_scenario_loading_attach_knock_out_and_bench_shrink() {
        let mut g = game();
        assert_occupied(&g);
        assert_eq!(g.st.players[0].in_play().count(), 3);
        let a = g.st.players[0].active;
        let (en, from) = find(&g, 0, ENERGY);
        g.move_card_to(from, en, ListRef::Slot(0, a));
        assert_occupied(&g);
        let b = g.st.players[0].bench.as_slice()[0];
        crate::engine::cards_zone::relocate_spot(&mut g, crate::effects::SlotRef::new(0, b), ListRef::Discard(0));
        assert_occupied(&g);
        assert_eq!(g.st.players[0].in_play().count(), 2);
        // A Bench spot gone from the Bench is no longer visited.
        g.st.players[0].bench.remove_at(1);
        assert_occupied(&g);
        assert_eq!(g.st.players[0].in_play().count(), 1);
        // A fresh spot is empty.
        let n = g.st.alloc_slot(0);
        assert_eq!(g.st.players[0].occupied >> n & 1, 0);
    }

    #[test]
    fn the_guard_recomputes_on_drop() {
        let mut g = game();
        let a = g.st.players[0].active;
        let (dura, _) = find(&g, 0, DURA);
        g.st.slot_edit(0, a).cards().clear();
        assert_eq!(g.st.slot_pokemon(0, a), None);
        g.st.slot_edit(0, a).cards().push(dura);
        assert_eq!(g.st.slot_pokemon(0, a), Some(dura));
        // A Pokemon card among the Tools is no top.
        let mut e = g.st.slot_edit(0, a);
        e.tools().push(dura);
        drop(e);
        assert_eq!(g.st.slot_pokemon(0, a), None);
        g.st.slot_edit(0, a).reset_keeping_cards();
        assert_eq!(g.st.slot_pokemon(0, a), Some(dura));
        assert_tops(&g);
    }

    /// The arrival stamps order every card that entered a spot, across spots: a card attached later, or moved to another
    /// Pokemon, has a larger stamp; a reorder of the stack and a swap of the spots don't restamp.
    #[test]
    fn arrival_stamps_follow_entering_a_spot() {
        let mut g = game();
        let a = g.st.players[0].active;
        let b = g.st.players[0].bench.as_slice()[0];
        let scenario_energy = g.st.slot(0, a).energies().as_slice()[0];
        let (e1, f1) = find(&g, 0, ENERGY);
        g.move_card_to(f1, e1, ListRef::Slot(0, b));
        let (e2, f2) = find(&g, 0, ENERGY);
        g.move_card_to(f2, e2, ListRef::Slot(0, a));
        let (e3, f3) = find(&g, 0, ENERGY);
        g.move_card_to(f3, e3, ListRef::Slot(0, b));
        let at = |g: &Game, c: CardId| g.st.cards[c as usize].arrived;
        assert!(0 < at(&g, scenario_energy) && at(&g, scenario_energy) < at(&g, e1) && at(&g, e1) < at(&g, e2) && at(&g, e2) < at(&g, e3));
        let tool = g.st.slot(0, a).tools().as_slice()[0];
        assert!(0 < at(&g, tool) && at(&g, tool) < at(&g, e1), "the Tool arrived while the scenario was loaded");
        // Moved to another Pokemon: it arrives there.
        let last = g.st.arrival;
        assert_eq!(last, at(&g, e3));
        g.move_card_to(ListRef::Slot(0, b), e1, ListRef::Slot(0, a));
        assert_eq!(at(&g, e1), last + 1);
        assert!(g.st.slot(0, a).energies().contains(e1) && !g.st.slot(0, b).energies().contains(e1));
        // A reorder of the stack doesn't restamp.
        let all = [scenario_energy, e1, e2, e3, tool];
        let before: Vec<u16> = all.iter().map(|c| at(&g, *c)).collect();
        let mut order: Vec<CardId> = g.st.slot(0, a).cards().iter().collect();
        order.reverse();
        *g.st.slot_edit(0, a).cards() = List::from_slice(&order);
        assert_eq!(before, all.iter().map(|c| at(&g, *c)).collect::<Vec<u16>>());
        // A swap of the spots (a retreat) moves the whole spot: no stamp changes.
        let n = g.st.arrival;
        g.st.players[0].active = b;
        g.st.players[0].bench.as_mut_slice()[0] = a;
        assert_eq!(g.st.arrival, n);
    }

    /// The Energy prompts list the attached cards of every Pokemon in arrival order, whatever the Pokemon's position, and
    /// each option still answers for its own card.
    #[test]
    fn the_energy_prompts_list_attached_cards_in_arrival_order() {
        use crate::prompts::{MoveOpts, PromptKind};
        let mut g = game_with(true);
        let a = g.st.players[0].active;
        let b = g.st.players[0].bench.as_slice()[0];
        let put = |g: &mut Game, s: SlotId| {
            let (e, f) = find(g, 0, ENERGY);
            g.move_card_to(f, e, ListRef::Slot(0, s));
            e
        };
        let e1 = put(&mut g, b);
        let e2 = put(&mut g, a);
        let e3 = put(&mut g, b);
        let scenario_energy = g.st.slot(0, a).energies().as_slice()[0];
        let mut slots = SVec::new();
        slots.push(crate::types::SlotType::Active as u8);
        slots.push(crate::types::SlotType::Bench as u8);
        let o = g.stash_opts(MoveOpts::default());
        g.prompt(
            0,
            crate::purpose::PromptPurpose::DiscardEnergy,
            PromptKind::DiscardEnergy { player_type: crate::types::PlayerType::BottomPlayer, slots, filter: crate::prompts::Filter::none(), o },
            crate::game::Cont::Noop,
        );
        let sel = g.select_with(false).unwrap().unwrap();
        let serials: Vec<CardId> = sel.options.iter().map(|o| o.serial.unwrap()).collect();
        assert_eq!(serials, vec![scenario_energy, e1, e2, e3], "the Active Spot holds the scenario Energy and e2, the Bench e1 and e3");
        for (k, c) in serials.iter().enumerate() {
            let res = g.decode_picks(&sel, &[k]).unwrap();
            let crate::prompts::Res::CardsFrom(picked) = res else { panic!("a card answer, got {:?}", res) };
            assert_eq!(picked.iter().map(|x| x.1).collect::<Vec<_>>(), vec![*c]);
        }
    }
}
