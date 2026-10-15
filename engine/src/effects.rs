//! Effects: one variant per kind of game event, carrying indices instead of object references. Effects live in the
//! store's arena and are addressed by [`EffId`] so continuations can share them.

use crate::cause::Cause;
use crate::list::*;
use crate::state::{AttackRef, ListRef, SlotId};
use crate::types::*;

pub type EffId = u8;

/// A board slot: (player index, slot arena id).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotRef {
    pub p: u8,
    pub s: SlotId,
}

impl SlotRef {
    pub const fn new(p: usize, s: SlotId) -> SlotRef {
        SlotRef { p: p as u8, s }
    }
    pub fn list(self) -> ListRef {
        ListRef::Slot(self.p, self.s)
    }
}

/// A power object: owning card instance and power index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerRef {
    pub card: CardId,
    pub index: u8,
}

/// What a lasting effect is put on (the ApplyEffect event): a Pokémon, or a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyTarget {
    Slot(SlotRef),
    Player(u8),
}

/// How a Pokémon leaves play (the LeavePlay event): a Knock Out's removal (APR D step 3: its Tools first, its effects and
/// Special Conditions with it), an effect's ("put it into your hand", "shuffle it into your deck", "discard it"), the
/// Bench shrinking below its Pokémon (the rule; the cards in today's order: the attached non-Pokémon cards, the Tools, the
/// Pokémon cards).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaveHow {
    KnockOut,
    Effect,
    BenchShrink,
    /// Cards attached to a Pokémon leave play, the Pokémon staying (an Energy or a Tool discarded, put into the hand or
    /// the deck; events batch 7, user decision D1: any card in play ceasing to be in play is a LeavePlay, the destination
    /// a consequence).
    Attached,
    /// The Stadium leaves play (discarded by an effect, or replaced by a new one, APR B-04; user decision D1).
    Stadium,
}

/// One pair of a MoveCounters event: HP of damage counters `removed` from the Pokémon in `from`, `placed` on the one in
/// `to` (0 when the destination is protected: they vanish). `i16`: an event carries up to `prompts::MAX_DAMAGE_RUNS`
/// pairs within the `Effect` size (ENGINE.md section 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterMove {
    pub from: SlotRef,
    pub to: SlotRef,
    pub removed: i16,
    pub placed: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnergyEntry {
    pub card: CardId,
    pub provides: SVec<CardType, 4>,
}

/// 64: a long stall can pile more than 32 Energy on one Pokémon (a deck holds 60 cards).
pub type EnergyMap = SVec<EnergyEntry, 64>;
/// Room for Rillaboom (TWM) mirrors: every copy in the game adds a [C].
pub type Cost = SVec<CardType, 16>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeaknessV {
    pub card_type: CardType,
    pub value: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResistanceV {
    pub card_type: CardType,
    pub value: i32,
}

/// Fields shared by every attack effect.
#[derive(Clone, Copy, Debug)]
pub struct AtkBase {
    /// The originating attack effect.
    pub attack_effect: EffId,
    pub player: u8,
    pub opponent: u8,
    pub attack: AttackRef,
    pub source: SlotRef,
    pub target: SlotRef,
    /// What caused the effect (events batch 1: filled at every construction, not read yet; the readers
    /// still infer "effect of an attack" from the presence of the `AtkBase`).
    pub cause: Cause,
}

/// The payloads too large for an [`Effect`] (batch 8 C8: an effect is copied on every read of the arena): a stack of
/// the effects in flight (nested events push above), indexed by the effect's `buf`. Pushed when the effect is made,
/// popped when its maker has read it back. (The check effects' Energy map, attack list and Weakness / Resistance went
/// with them in the batch 8 closeout.)
#[derive(Clone, Copy, Default)]
pub struct EffectPayloads {
    pub moves: SVec<SVec<CounterMove, { crate::prompts::MAX_DAMAGE_RUNS }>, 4>,
}

impl EffectPayloads {
    /// Copy the live entries of `src` into `dst` (`Game::copy_from`).
    ///
    /// # Safety
    /// `dst` must be valid for writes and properly aligned.
    pub unsafe fn copy_live_to(&self, dst: *mut Self) {
        use std::ptr::addr_of_mut as f;
        self.moves.copy_live_to(f!((*dst).moves));
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    // ---- game phase ----
    BeginTurn { p: u8 },
    EndTurn { p: u8 },
    BetweenTurns { p: u8, poison_damage: i32, burn_damage: i32 },
    /// `atk` is the attack's own AttackEffect : effect text asked after the damage keeps
    /// the attack's state through it.
    AfterAttack { p: u8, opp: u8, attack: AttackRef, atk: EffId },
    /// Sent once everything the attack did (AfterAttack, prompts included) is resolved.
    AfterAttackTriggers { p: u8, opp: u8, attack: AttackRef },
    BeforeDoingDamage { attack_effect: EffId, p: u8, opp: u8, attack: AttackRef },

    // ---- game ----
    Retreat { p: u8, ignore_status_conditions: bool, move_retreat_cost_to: ListRef },
    RetreatStart { p: u8 },
    /// `delegate_from`: the copy-attack source card.
    /// `second_attack`: the second attack of the turn of a Pokémon that may attack again (Festival Lead); no third is offered.
    UseAttack { p: u8, attack: AttackRef, source: SlotRef, ignore_status_conditions: bool, second_attack: bool, delegate_from: Option<CardId> },
    UseStadium { p: u8, stadium: CardId },
    UsePower { p: u8, power: PowerRef, card: CardId, target: CardTarget, bench_target: Option<SlotRef> },
    Power { p: u8, power: PowerRef, card: CardId, target: Option<SlotRef> },
    /// `ignore_defender_effects`: (Shred: effects on the damaged
    /// Pokémon don't change this attack's damage; see `prefabs::ignores_defender_effects`).
    Attack { p: u8, opp: u8, attack: AttackRef, damage: i32, ignore_weakness: bool, ignore_resistance: bool, ignore_defender_effects: bool, source: SlotRef, second_attack: bool },
    /// The KnockOut event (events batch 6; `engine::knockout`): the Pokémon in `target` of player `p` is Knocked Out
    /// (`ko_by`: by damage from an attack, by an effect, otherwise; `cause` the attack, the effect, or the rule), worth
    /// `prize_count` Prize cards to the opponent (`prize_base` before the adjustments that reduce it). Produced by the
    /// state check while every Pokémon Knocked Out at the same time is still in play.
    KnockOut { p: u8, target: SlotRef, prize_count: i32, prize_base: i32, ko_by: crate::spec::event::KoBy, cause: Cause },
    /// The LeavePlay event (`engine::knockout`): a card in play ceases to be in play (user decision D1), for `dest` (its
    /// owner's zone). `how`: the Pokémon in `target` and every card attached to it (a Knock Out's, an effect's, the Bench
    /// shrinking: `cards` is empty), the `cards` attached to the Pokémon in `target` (`Attached`), the Stadium `cards[0]`
    /// (`Stadium`, no `target`). `p` is the owner of what leaves. `pokemon`: for a whole Pokémon, its top Pokémon card as
    /// it leaves (what the triggers after the event read); `NO_CARD` otherwise.
    LeavePlay { p: u8, target: Option<SlotRef>, pokemon: CardId, dest: ListRef, cause: Cause, how: LeaveHow, source_card: CardId, cards: SVec<CardId, 64> },
    /// The TakePrizes event (`engine::knockout::take_prizes_chosen`): player `p` takes the Prize cards `prizes` (Prize list
    /// indices, in the order taken) into their hand.
    TakePrizes { p: u8, prizes: crate::list::SVec<u8, 6>, cause: Cause },
    /// The RemoveCounters event (events batch 4; `engine::condition::heal`): `damage` HP of damage counters are
    /// removed from the Pokémon in `target` (healing; APR C-06). Every heal produces it (an attack's, a Trainer's,
    /// an Ability's). `p` is the Pokémon's owner.
    Heal { p: u8, target: SlotRef, damage: i32, cause: Cause },
    /// The GainCondition event (events batch 4; `engine::condition::gain`): the Pokémon in `target` is now affected
    /// by `condition`. `counters`: the damage counters its rule places (Poison 1 at each Checkup, Burn 2, Confusion
    /// 3 when the attack fails; 0 for Asleep and Paralyzed): the base, plus any "more" an effect adds. `p` is the
    /// Pokémon's owner.
    GainCondition { p: u8, target: SlotRef, condition: crate::types::SpecialCondition, counters: u8, cause: Cause },
    /// The RemoveCondition event (`engine::condition::remove`): the Pokémon in `target` recovers from `condition`
    /// (an effect, evolving or devolving, moving to the Bench, the Checkup).
    RemoveCondition { p: u8, target: SlotRef, condition: crate::types::SpecialCondition, cause: Cause },
    /// The CoinFlip event (`engine::condition::coin_flipped`): player `p` flipped a coin, `heads` or not, for
    /// `purpose` (an effect, Confusion, Burned, Asleep, who goes first). Produced once per physical flip, when its
    /// result is known; a card's flips are made by `engine::condition::flip_coin` / `flip_sequence`, the others by a
    /// Checkup / Confusion prompt.
    CoinFlip { p: u8, purpose: crate::spec::event::CoinPurpose, heads: bool, cause: Cause },
    /// The Evolve event (events batch 2; `engine::enter::evolve`): every evolution, played from the hand, Rare
    /// Candy, and the effects that evolve from the deck or elsewhere. `from` is the list the card physically
    /// leaves, `source` its rules zone (what "from your hand" reads), `base` the Pokémon evolved from;
    /// `base_entered_this_turn` / `owner_first_turn` as they were before the event (the limits it was checked
    /// against: `EventView`).
    Evolve { p: u8, target: SlotRef, card: CardId, base: CardId, from: ListRef, source: crate::spec::event::RulesZone, path: crate::spec::event::EvolvePath, cause: Cause, base_entered_this_turn: bool, owner_first_turn: bool },
    /// The EnterPlay event (`engine::enter::enter_play`): a Pokémon card goes onto the empty spot `target` of
    /// its owner `p`: played from the hand by the rule, put by an effect, or set up.
    EnterPlay { p: u8, card: CardId, target: SlotRef, from: ListRef, source: crate::spec::event::RulesZone, mode: crate::spec::event::EnterMode, cause: Cause },
    /// The Devolve event (`engine::enter::devolve`): the top `count` Evolution cards of the Pokémon leave it for `dest`
    /// (one event per devolving action, user decision D6); `removed` is filled by the reducer (highest Stage first).
    Devolve { p: u8, target: SlotRef, count: u8, removed: SVec<CardId, 3>, dest: ListRef, cause: Cause },
    /// The Swap event (`engine::enter::swap`): the Pokémon card `old` in `target` was replaced by `new`, which came
    /// from `source` (its rules zone before the swap).
    Swap { p: u8, target: SlotRef, old: CardId, new: CardId, source: crate::spec::event::RulesZone, cause: Cause, from: ListRef, into: ListRef, place: crate::engine::enter::SwapPlace, by: CardId },
    /// The ApplyEffect event (events batch 6, user decision D4; `engine::apply`): player `p`'s attack `attack` (used by
    /// `card`) puts the lasting `effect` on `target`.
    ApplyEffect { p: u8, target: ApplyTarget, effect: crate::spec::ops::state::Lasting, card: CardId, attack: Option<AttackRef>, cause: Cause },
    /// The Discard event (events batch 7; `engine::cards_zone::discard`): player `p`'s `cards` go from `from` (their hand,
    /// their deck, or the cards looked at, `source` the rules zone) to their discard pile (a Prism Star card to the Lost
    /// Zone), in the order of the action. One per action and owner.
    Discard { p: u8, cards: SVec<CardId, 64>, from: ListRef, source: crate::spec::event::RulesZone, cause: Cause },
    /// The PutIntoHand event (`engine::cards_zone::put_into_hand`): player `p`'s `cards` go from `from` into their hand.
    PutIntoHand { p: u8, cards: SVec<CardId, 64>, from: ListRef, source: crate::spec::event::RulesZone, cause: Cause },
    /// The PutIntoDeck event (`engine::cards_zone::put_into_deck`): player `p`'s `cards` go from `from` into their deck at
    /// `position`.
    PutIntoDeck { p: u8, cards: SVec<CardId, 64>, from: ListRef, source: crate::spec::event::RulesZone, position: crate::spec::event::DeckPosition, cause: Cause },
    /// The Draw event (`engine::cards_zone::draw`): player `p` draws `cards` (the top of their deck) into their hand.
    Draw { p: u8, cards: SVec<CardId, 64>, cause: Cause },
    /// The PlaceCounters event (events batch 6; `engine::damage::place`): `amount` HP of damage counters are put on the
    /// Pokémon in `target` (APR C-07), whatever causes it. `p` is the Pokémon's owner.
    PlaceCounters { p: u8, target: SlotRef, amount: i32, cause: Cause },
    /// The MoveCounters event (events batch 6; `engine::damage::move_counters`): one action moving damage counters between
    /// Pokémon, its pairs as they happened. `p` is the player whose effect moves them.
    /// `buf`: its pairs (`Game::fxp.moves`).
    MoveCounters { p: u8, buf: u8, cause: Cause },
    /// The ChangeActive event (events batch 5; `engine::change_active`): player `p`'s Active Pokémon changes: the
    /// Pokémon in the Active Spot `from` (none for a promotion) goes to the Bench and the Benched Pokémon in `to`
    /// becomes the Active Pokémon, as `change` says (retreat, switch, switch-in, switch-out, promotion).
    ChangeActive { p: u8, from: crate::state::SlotId, to: crate::state::SlotId, change: crate::spec::event::ActiveChange, cause: Cause },

    // ---- attack sub-effects ----
    /// The Damage event (events batch 6; `engine::damage::deal`): the attack of `b` puts `amount` damage on the Pokémon
    /// in `b.target`, after the whole calculation and its preventions (APR A-01 step 6). `survive`: a survive-on-10
    /// replacement armed in the calculation.
    Damage { b: AtkBase, amount: i32, survive: bool },

    // ---- play card ----
    /// The Attach event (events batch 3; `engine::attach`): an Energy or a Pokémon Tool card goes onto the Pokémon
    /// in `target` from a zone that isn't a Pokémon (the hand, the deck, the discard pile, the cards looked at).
    /// `from` is the list the card physically leaves, `source` its rules zone (what "from your hand" reads),
    /// `manual` the turn's one Energy attachment by the game rule (nothing else is manual; effects don't use it
    /// up, APR C-09). `p` is the card's owner.
    Attach { p: u8, card: CardId, target: SlotRef, from: ListRef, source: crate::spec::event::RulesZone, manual: bool, cause: Cause },
    /// The MoveEnergy event (`engine::attach::move_energy`): an attached Energy card moves from the Pokémon in
    /// `from` to the one in `to` (APR C-10; not an Attach, id1653). `p` is the Pokémon's owner.
    MoveEnergy { p: u8, card: CardId, from: SlotRef, to: SlotRef, cause: Cause },
    /// The MoveTool event (`engine::attach::move_tool`): an attached Pokémon Tool moves between Pokémon.
    MoveTool { p: u8, card: CardId, from: SlotRef, to: SlotRef, cause: Cause },
    /// The PlayTrainer event (events batch 7; `engine::play_trainer`): player `p` plays the Trainer `card` from the hand
    /// (`use_: Played`; `target` a Tool's Pokémon) or uses its effect as the effect of an attack (`Used`: Mr. Mime's
    /// Look-Alike Show; id2225, id2226, id2376). The card's own program is its handler (`spec::run::reduce`).
    /// `with`: the companion played together with it ("you must play 2 X cards at once", `CardSpec::together`).
    PlayTrainer { p: u8, card: CardId, target: Option<SlotRef>, use_: crate::spec::event::TrainerUse, cause: Cause, with: Option<CardId> },
}

impl Effect {
    /// The effect kind's name in the recorded traces.
    pub fn type_name(&self) -> &'static str {
        use Effect::*;
        match self {
            BeginTurn { .. } => "BEGIN_TURN_EFFECT",
            EndTurn { .. } => "END_TURN_EFFECT",
            BetweenTurns { .. } => "BETWEEN_TURNS_EFFECT",
            AfterAttack { .. } => "AFTER_ATTACK_EFFECT",
            AfterAttackTriggers { .. } => "AFTER_ATTACK_TRIGGERS_EFFECT",
            BeforeDoingDamage { .. } => "BEFORE_DOING_DAMAGE_EFFECT",
            Retreat { .. } => "RETREAT_EFFECT",
            RetreatStart { .. } => "RETREAT_START_EFFECT",
            UseAttack { .. } => "USE_ATTACK_EFFECT",
            UseStadium { .. } => "USE_STADIUM_EFFECT",
            UsePower { .. } => "USE_POWER_EFFECT",
            Power { .. } => "POWER_EFFECT",
            Attack { .. } => "ATTACK_EFFECT",
            KnockOut { .. } => "KNOCK_OUT_EVENT",
            LeavePlay { .. } => "LEAVE_PLAY_EVENT",
            TakePrizes { .. } => "TAKE_PRIZES_EVENT",
            Heal { .. } => "HEAL_EFFECT",
            GainCondition { .. } => "GAIN_CONDITION_EVENT",
            RemoveCondition { .. } => "REMOVE_CONDITION_EVENT",
            CoinFlip { .. } => "COIN_FLIP_EVENT",
            Evolve { .. } => "EVOLVE_EFFECT",
            Discard { .. } => "DISCARD_EVENT",
            PutIntoHand { .. } => "PUT_INTO_HAND_EVENT",
            PutIntoDeck { .. } => "PUT_INTO_DECK_EVENT",
            Draw { .. } => "DRAW_EVENT",
            PlaceCounters { .. } => "PLACE_COUNTERS_EVENT",
            ChangeActive { .. } => "CHANGE_ACTIVE_EVENT",
            Damage { .. } => "DAMAGE_EVENT",
            MoveCounters { .. } => "MOVE_COUNTERS_EVENT",
            Devolve { .. } => "DEVOLVE_EVENT",
            Swap { .. } => "SWAP_EVENT",
            ApplyEffect { .. } => "APPLY_EFFECT_EVENT",
            Attach { .. } => "ATTACH_EVENT",
            MoveEnergy { .. } => "MOVE_ENERGY_EVENT",
            MoveTool { .. } => "MOVE_TOOL_EVENT",
            EnterPlay { .. } => "ENTER_PLAY_EVENT",
            PlayTrainer { .. } => "PLAY_TRAINER_EVENT",
        }
    }

    pub fn atk_base(&self) -> Option<&AtkBase> {
        use Effect::*;
        match self {
            Damage { b, .. } => Some(b),
            _ => None,
        }
    }

    /// Subscription class used to skip cards that never react to an effect.
    pub fn kind(&self) -> u32 {
        use Effect::*;
        let k = match self {
            BeginTurn { .. } => 0,
            EndTurn { .. } => 3,
            BetweenTurns { .. } => 5,
            AfterAttack { .. } => 6,
            AfterAttackTriggers { .. } => 245,
            BeforeDoingDamage { .. } => 7,
            Retreat { .. } => 20,
            RetreatStart { .. } => 21,
            UseAttack { .. } => 22,
            UseStadium { .. } => 23,
            UsePower { .. } => 24,
            Power { .. } => 25,
            Attack { .. } => 26,
            KnockOut { .. } => 27,
            LeavePlay { .. } => 98,
            TakePrizes { .. } => 97,
            Heal { .. } => 28,
            GainCondition { .. } => 73,
            RemoveCondition { .. } => 74,
            CoinFlip { .. } => 75,
            Evolve { .. } => 29,
            Discard { .. } => 95,
            PutIntoHand { .. } => 159,
            PutIntoDeck { .. } => 127,
            Draw { .. } => 130,
            PlaceCounters { .. } => 100,
            ChangeActive { .. } => 48,
            Damage { .. } => 94,
            Attach { .. } => 50,
            MoveEnergy { .. } => 64,
            MoveTool { .. } => 65,
            EnterPlay { .. } => 51,
            PlayTrainer { .. } => 56,
            MoveCounters { .. } => 129,
            Devolve { .. } => 248,
            Swap { .. } => 249,
            ApplyEffect { .. } => 116,
        };
        k
    }
}

/// `Effect::kind()` values, for subscription masks.
pub mod k {
    pub const BEGIN_TURN: u32 = 0;
    pub const END_TURN: u32 = 3;
    pub const BETWEEN_TURNS: u32 = 5;
    pub const AFTER_ATTACK: u32 = 6;
    pub const AFTER_ATTACK_TRIGGERS: u32 = 245;
    pub const BEFORE_DOING_DAMAGE: u32 = 7;
    pub const RETREAT: u32 = 20;
    pub const RETREAT_START: u32 = 21;
    pub const USE_ATTACK: u32 = 22;
    pub const USE_STADIUM: u32 = 23;
    pub const USE_POWER: u32 = 24;
    pub const POWER: u32 = 25;
    pub const ATTACK: u32 = 26;
    pub const KNOCK_OUT: u32 = 27;
    /// The RemoveCounters event (healing; events batch 4).
    pub const HEAL: u32 = 28;
    /// The GainCondition / RemoveCondition / CoinFlip events (events batch 4).
    pub const GAIN_CONDITION: u32 = 73;
    pub const REMOVE_CONDITION: u32 = 74;
    pub const COIN_FLIP: u32 = 75;
    pub const EVOLVE: u32 = 29;
    /// The ChangeActive event (events batch 5). 48, not the old MovedToActive's 35: the dispatch index keys its
    /// entries by `kind % 32` (`dispatch::SLOTS`), and 35 shared its entry with END_TURN (3), which games with a
    /// ChangeActive handler (every attack-effect protection) then rebuilt in turn; nothing else uses entry 16.
    pub const CHANGE_ACTIVE: u32 = 48;
    /// The Attach event (events batch 3); the number the old AttachEnergy effect had.
    pub const ATTACH: u32 = 50;
    pub const MOVE_ENERGY: u32 = 64;
    pub const MOVE_TOOL: u32 = 65;
    pub const ENTER_PLAY: u32 = 51;
    /// The PlayTrainer event (events batch 7): the number the Trainer effect had (the Trainer's program is its handler).
    pub const PLAY_TRAINER: u32 = 56;
    /// The Devolve and Swap events (events batch 2).
    pub const DEVOLVE: u32 = 248;
    pub const SWAP: u32 = 249;
    /// The events of events batch 6 (`engine::damage`, `engine::knockout`; KnockOut keeps `KNOCK_OUT`): Damage, PlaceCounters,
    /// MoveCounters, LeavePlay, TakePrizes, and ApplyEffect (a lasting effect put on a Pokémon or a player, user decision D4).
    /// The dispatch index keys its entries by `kind % 32` (`dispatch::SLOTS`); the 32 entries are all shared, so each
    /// number takes an entry whose other kinds have no card listeners, else only listeners of rare events (ENGINE.md
    /// section 12; the counts: `dispatch::tests::listener_table`): Damage 30 (TrainerTarget: none), PlaceCounters 4
    /// (WhoBegins: none), ApplyEffect 20 (Retreat, PlaySupporter: none), TakePrizes 1 (it has no
    /// listener and is never looked up), MoveCounters 1 (MoveTool: rare), LeavePlay 2 (shared with Draw since events batch 7,
    /// which removed CoinFlipSequence). MoveCounters was 117 and LeavePlay 119 (shared with AfterAttackTriggers / PlayStadium and
    /// UseStadium / PlayItem).
    pub const DAMAGE: u32 = 94;
    pub const PLACE_COUNTERS: u32 = 100;
    pub const MOVE_COUNTERS_EVENT: u32 = 129;
    pub const LEAVE_PLAY: u32 = 98;
    pub const TAKE_PRIZES: u32 = 97;
    pub const APPLY_EFFECT: u32 = 116;
    /// The view of a continuous effect of an Ability done to a Pokémon (batch 8 C11): no effect is made, the derived
    /// layer asks the `Prevent` reader with it.
    pub const CONTINUOUS_EFFECT: u32 = 222;
    /// A `Prevent` over continuous effects (a kind-free one: Hide 'n' Sneak).
    pub const DECLARES_CONTINUOUS_PREVENT: u32 = 223;
    /// The card-movement events of events batch 7 (`engine::cards_zone`): Discard (cards from the hand or the deck),
    /// PutIntoHand, PutIntoDeck, Draw. Their dispatch-index entries (`kind % 32`) are ones no hot kind uses: Discard,
    /// PutIntoDeck and PutIntoHand 31 (MoveCards, which they replace; otherwise only the marker PERMIT_EVOLVES_FROM), Draw 2
    /// (with LeavePlay; DrewTopdeck and CoinFlipSequence, its old sharers, are gone).
    pub const DISCARD: u32 = 95;
    pub const PUT_INTO_HAND: u32 = 159;
    pub const PUT_INTO_DECK: u32 = 127;
    pub const DRAW: u32 = 130;
    /// A lock over Discard / PutIntoHand / PutIntoDeck / Draw / LeavePlay (events batch 7: Poké Vital A's and Neutralization
    /// Zone's "this card can't be put into your hand or deck from the discard pile", Milotic TWM 50's "can't be put into your
    /// opponent's hand").
    pub const DECLARES_CARD_LOCK: u32 = 218;
    /// A lock over PlayTrainer (events batch 7: "your opponent can't play Item / Supporter / Stadium cards from their hand").
    pub const DECLARES_PLAY_LOCK: u32 = 219;
    /// A lock over UseAttack / UseAbility / UseStadium (events batch 7: "this Pokémon can't attack unless ...").
    pub const DECLARES_USE_LOCK: u32 = 220;
    /// A re-flip declaration (`Modifier::Reflip`: Backtrack Badge), read by `passive::reflip_offered`.
    pub const DECLARES_REFLIP: u32 = 221;
    /// A `Prevent` declaration over PlaceCounters / MoveCounters, over Damage, over a KnockOut by an effect, over LeavePlay,
    /// over Attach / MoveEnergy / MoveTool, over Evolve / Devolve / Swap, over ApplyEffect (events batch 6: the event's
    /// routine asks `derived::event_prevented` only in a game with one).
    pub const DECLARES_COUNTER_PREVENT: u32 = 210;
    pub const DECLARES_DAMAGE_PREVENT: u32 = 211;
    pub const DECLARES_KO_PREVENT: u32 = 212;
    pub const DECLARES_LEAVE_PREVENT: u32 = 213;
    pub const DECLARES_ATTACH_PREVENT: u32 = 214;
    pub const DECLARES_POKEMON_PREVENT: u32 = 215;
    pub const DECLARES_APPLY_PREVENT: u32 = 217;
    /// A lock over PlaceCounters / MoveCounters (Patrat's Watchful Eye; events batch 6).
    pub const DECLARES_COUNTER_LOCK: u32 = 216;
    // Declaration markers (never dispatched): set in a card's mask when it declares a permission, a
    // restriction or rule limits (`CardSpec::restricts` / `limits`) or a lock over events, so `Game::kinds_present` says whether a game has any.
    pub const DECLARES_PERMIT: u32 = 250;
    pub const DECLARES_RESTRICT: u32 = 251;
    /// A lock over the Pokémon events (EnterPlay, Evolve, Devolve, Swap).
    pub const DECLARES_EVENT_LOCK: u32 = 252;
    /// A lock over the attaching events (Attach, MoveEnergy, MoveTool; events batch 3).
    pub const DECLARES_ATTACH_LOCK: u32 = 71;
    /// An Ability lock whose taking hold can depend on what is attached to a Pokémon: its spot predicate reads
    /// the attached cards (directly, or through a checked read such as the type, which attached cards can
    /// change), its probe goes through the Stadium's effect on the spot, or its source's own Ability has a
    /// program (`spec::passive::lock_reads_attached`). Without one, an Attach / MoveEnergy / MoveTool can't
    /// change the take-hold stamps (`spec::passive::lock_sync_attached`).
    pub const DECLARES_ATTACHED_LOCK: u32 = 72;
    /// A lock over GainCondition / RemoveCondition, over RemoveCounters (healing), over CoinFlip (events batch 4).
    pub const DECLARES_CONDITION_LOCK: u32 = 76;
    pub const DECLARES_HEAL_LOCK: u32 = 78;
    pub const DECLARES_COIN_LOCK: u32 = 79;
    /// A `Prevent` declaration over GainCondition / RemoveCondition, over RemoveCounters, over CoinFlip (events
    /// batch 4: the event's routine asks `derived::event_prevented` only in a game with one).
    pub const DECLARES_CONDITION_PREVENT: u32 = 80;
    pub const DECLARES_HEAL_PREVENT: u32 = 81;
    pub const DECLARES_COIN_PREVENT: u32 = 82;
    /// An Ability lock (`AbilityLock`, Midnight Fluttering, Initialization): the take-hold stamps are kept and legality reads
    /// the powers a Pokémon has now only in a game with one (batch 8 closeout: the CheckPokemonPowers kind it replaces).
    pub const DECLARES_ABILITY_LOCK: u32 = 14;
    /// A lock / a `Prevent` declaration over ChangeActive (events batch 5).
    pub const DECLARES_ACTIVE_LOCK: u32 = 36;
    pub const DECLARES_ACTIVE_PREVENT: u32 = 45;
    /// A permission that lifts `Limit::FirstTurn` / `BaseEnteredThisTurn` / `EvolvesFrom` (with `DECLARES_PERMIT`).
    pub const PERMIT_FIRST_TURN: u32 = 253;
    pub const PERMIT_BASE_ENTERED: u32 = 254;
    pub const PERMIT_EVOLVES_FROM: u32 = 255;
    /// A permission that lifts `Limit::GoingFirst` (Carmine, Team Rocket's Proton; with `DECLARES_PERMIT`).
    pub const PERMIT_GOING_FIRST: u32 = 224;
}

/// Build a subscription mask: `mask(&[k::ATTACK, k::TRAINER])`.
pub const fn mask(kinds: &[u32]) -> KindMask {
    let mut m = [0u64; 4];
    let mut i = 0;
    while i < kinds.len() {
        let k = kinds[i];
        m[(k >> 6) as usize] |= 1u64 << (k & 63);
        i += 1;
    }
    KindMask(m)
}

/// Bitmask over [`Effect::kind`] (256 kinds). Combine in const context with
/// [`KindMask::or`]; `|` works at runtime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KindMask(pub [u64; 4]);

impl KindMask {
    pub const EMPTY: KindMask = KindMask([0; 4]);

    #[inline]
    pub const fn has(&self, kind: u32) -> bool {
        (self.0[(kind >> 6) as usize] >> (kind & 63)) & 1 != 0
    }

    pub const fn or(self, o: KindMask) -> KindMask {
        KindMask([self.0[0] | o.0[0], self.0[1] | o.0[1], self.0[2] | o.0[2], self.0[3] | o.0[3]])
    }

    /// Do the two masks share a kind?
    pub const fn intersects(self, o: KindMask) -> bool {
        (self.0[0] & o.0[0]) | (self.0[1] & o.0[1]) | (self.0[2] & o.0[2]) | (self.0[3] & o.0[3]) != 0
    }
}

impl std::ops::BitOr for KindMask {
    type Output = KindMask;
    fn bitor(self, o: KindMask) -> KindMask {
        self.or(o)
    }
}

impl std::ops::BitOrAssign for KindMask {
    fn bitor_assign(&mut self, o: KindMask) {
        *self = self.or(o);
    }
}

pub const ALL_KINDS: KindMask = KindMask([u64::MAX; 4]);

impl Effect {
    /// The effect's `Cause`, for the effects that carry one (events batch 1).
    pub fn cause(&self) -> Option<Cause> {
        use Effect::*;
        if let Some(b) = self.atk_base() {
            return Some(b.cause);
        }
        match *self {
            ChangeActive { cause, .. }
            | Heal { cause, .. }
            | Evolve { cause, .. }
            | PlaceCounters { cause, .. }
            | MoveCounters { cause, .. }
            | KnockOut { cause, .. }
            | LeavePlay { cause, .. }
            | PlayTrainer { cause, .. }
            | Discard { cause, .. }
            | PutIntoHand { cause, .. }
            | PutIntoDeck { cause, .. }
            | Draw { cause, .. }
            | TakePrizes { cause, .. }
            | ApplyEffect { cause, .. }
            | GainCondition { cause, .. }
            | RemoveCondition { cause, .. }
            | CoinFlip { cause, .. }
            | Attach { cause, .. }
            | MoveEnergy { cause, .. }
            | MoveTool { cause, .. }
            | EnterPlay { cause, .. }
            | Devolve { cause, .. }
            | Swap { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

#[cfg(test)]
mod marker_tests {
    //! The declaration markers (`k::DECLARES_*`, `k::PERMIT_*`) share the mask with the effect kinds; some reuse
    //! numbers effects freed (36, 45, 71, ...). A marker equal to an effect kind would alias: dispatching that effect
    //! would call every card that declares the marker, and `kinds_present` would say a declaration exists whenever a
    //! card reacts to the effect.

    /// The numbers `Effect::kind()` returns and the markers' numbers, read from this file.
    fn numbers() -> (Vec<u32>, Vec<(String, u32)>) {
        let src = include_str!("effects.rs");
        let body = src.split("pub fn kind(&self) -> u32 {").nth(1).unwrap().split("        };").next().unwrap();
        let kinds: Vec<u32> = body.lines().filter_map(|l| l.trim().strip_suffix(',')?.rsplit("=> ").next()?.parse().ok()).collect();
        let markers = src
            .lines()
            .filter_map(|l| {
                let l = l.trim().strip_prefix("pub const ")?;
                let (name, v) = l.split_once(": u32 = ")?;
                (name.starts_with("DECLARES_") || name.starts_with("PERMIT_")).then(|| (name.to_string(), v.trim_end_matches(';').parse().unwrap()))
            })
            .collect();
        (kinds, markers)
    }

    #[test]
    fn no_marker_is_an_effect_kind() {
        let (kinds, markers) = numbers();
        // (Batch 8 removed the probes, the table-state checks and, in the closeout, the check effects: 38 kinds left.)
        assert!(kinds.len() >= 36, "every Effect variant's kind is read ({})", kinds.len());
        assert!(markers.len() >= 16, "the markers are read ({})", markers.len());
        for (name, v) in &markers {
            assert!(!kinds.contains(v), "k::{name} = {v} is also an effect kind");
            assert!(markers.iter().filter(|(_, w)| w == v).count() == 1, "k::{name} = {v} is used twice");
        }
        assert!(markers.iter().any(|(n, v)| n == "DECLARES_ACTIVE_LOCK" && *v == super::k::DECLARES_ACTIVE_LOCK));
        assert!(markers.iter().any(|(n, v)| n == "DECLARES_ACTIVE_PREVENT" && *v == super::k::DECLARES_ACTIVE_PREVENT));
    }
}
