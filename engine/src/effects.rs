//! Effects: one variant per Twinleaf effect class (same names), carrying
//! indices instead of object references. Effects live in the store's arena
//! and are addressed by [`EffId`] so continuations can share them the way
//! Twinleaf closures share effect objects.

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnergyEntry {
    pub card: CardId,
    pub provides: SVec<CardType, 4>,
}

/// 64: a long stall can pile more than 32 Energy on one Pokémon (Twinleaf has no cap; a deck holds 60 cards).
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

/// Fields shared by every `AbstractAttackEffect`.
#[derive(Clone, Copy, Debug)]
pub struct AtkBase {
    /// The originating `AttackEffect`.
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

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    // ---- game phase ----
    BeginTurn { p: u8 },
    DrawCardForTurn { p: u8, draw_count: i32 },
    DrewTopdeck { p: u8, card: CardId },
    EndTurn { p: u8 },
    WhoBegins { player: Option<u8> },
    BetweenTurns { p: u8, poison_damage: i32, burn_damage: i32, burn_flip_result: Option<bool>, asleep_flip_result: Option<bool> },
    /// `atk` is the attack's own AttackEffect (`AfterAttackEffect.attackEffect`): effect text asked after the damage keeps
    /// the attack's state through it.
    AfterAttack { p: u8, opp: u8, attack: AttackRef, atk: EffId },
    /// Sent once everything the attack did (AfterAttack, prompts included) is resolved.
    AfterAttackTriggers { p: u8, opp: u8, attack: AttackRef },
    BeforeDoingDamage { attack_effect: EffId, p: u8, opp: u8, attack: AttackRef },

    // ---- checks ----
    CheckHp { p: u8, target: SlotRef, card: Option<CardId> },
    CheckPokemonStats { target: SlotRef, weakness: SVec<WeaknessV, 4>, resistance: SVec<ResistanceV, 4> },
    CheckPokemonType { target: SlotRef, card_types: SVec<CardType, 4> },
    CheckRetreatCost { p: u8, cost: Cost, no_cost: bool, reduction: u8 },
    /// `set_cost` / `ignore_colorless`: an effect that sets or ignores the cost
    /// (Kyurem's Plasma Bane, Conkeldurr, ...); applied after all handlers.
    /// `reduction`: "costs [C] less" effects add up and `any_reduction` (Sparkling Crystal, "1 Energy less") is applied once
    /// after all handlers, with the increases (Advanced Rulebook D-11, D-12).
    CheckAttackCost { p: u8, attack: AttackRef, cost: Cost, set_cost: Option<Cost>, ignore_colorless: bool, reduction: u8, any_reduction: bool },
    CheckProvidedEnergy { p: u8, source: SlotRef, energy_map: EnergyMap },
    CheckPokemonPowers { p: u8, target: CardId, powers: SVec<PowerRef, 8> },
    /// `copied`: the attacks of other Pokémon that the Active Pokémon uses as its own (Mew ex Memory Helix),
    /// pushed to `attacks` as well; the source is `AttackRef::card`.
    CheckPokemonAttacks { p: u8, attacks: SVec<AttackRef, 32>, copied: SVec<AttackRef, 32> },
    CheckTableState { bench_sizes: [u8; 2] },
    CheckPrizesDestination { p: u8, destination: ListRef },
    CheckSpecialConditionRemoval { p: u8, target: SlotRef, preserved: SVec<u8, 5> },

    // ---- game ----
    Retreat { p: u8, bench_index: u8, ignore_status_conditions: bool, move_retreat_cost_to: ListRef },
    RetreatStart { p: u8 },
    /// `delegate_from`: `UseAttackEffect.delegateFrom` (copy-attack source card).
    UseAttack { p: u8, attack: AttackRef, source: SlotRef, ignore_status_conditions: bool, barrage_used: bool, delegate_from: Option<CardId> },
    UseStadium { p: u8, stadium: CardId },
    UsePower { p: u8, power: PowerRef, card: CardId, target: CardTarget, bench_target: Option<SlotRef> },
    /// `probe`: the lock-check stand-in power ('test') carrying the real power's flags.
    Power { p: u8, power: PowerRef, card: CardId, target: Option<SlotRef>, probe: bool },
    /// `ignore_defender_effects`: `AttackEffect.ignoreDefenderEffects` (Shred: effects on the damaged
    /// Pokémon don't change this attack's damage; see `prefabs::ignores_defender_effects`).
    Attack { p: u8, opp: u8, attack: AttackRef, damage: i32, ignore_weakness: bool, ignore_resistance: bool, ignore_defender_effects: bool, source: SlotRef, barrage_used: bool },
    /// `defer_removal`: the Check State step announces every Knock Out first and takes the
    /// Pokémon out of play later (`game_effect::complete_knock_out`).
    KnockOut { p: u8, target: SlotRef, prize_count: i32, prize_base: i32, prize_destination: Option<ListRef>, attack: Option<AttackRef>, defer_removal: bool },
    Heal { p: u8, target: SlotRef, damage: i32, cause: Cause },
    /// The Evolve event (events batch 2; `engine::enter::evolve`): every evolution, played from the hand, Rare
    /// Candy, and the effects that evolve from the deck or elsewhere. `from` is the list the card physically
    /// leaves, `source` its rules zone (what "from your hand" reads), `base` the Pokémon evolved from.
    Evolve { p: u8, target: SlotRef, card: CardId, base: CardId, from: ListRef, source: crate::spec::event::RulesZone, path: crate::spec::event::EvolvePath, cause: Cause },
    /// The EnterPlay event (`engine::enter::enter_play`): a Pokémon card goes onto the empty spot `target` of
    /// its owner `p`: played from the hand by the rule, put by an effect, or set up.
    EnterPlay { p: u8, card: CardId, target: SlotRef, from: ListRef, source: crate::spec::event::RulesZone, mode: crate::spec::event::EnterMode, cause: Cause },
    /// The Devolve event (`engine::enter::devolve`): `removed` (highest Stage first) left the Pokémon for `dest`.
    Devolve { p: u8, target: SlotRef, removed: SVec<CardId, 3>, dest: ListRef, cause: Cause },
    /// The Swap event (`engine::enter::swap`): the Pokémon card `old` in `target` was replaced by `new`.
    Swap { p: u8, target: SlotRef, old: CardId, new: CardId, cause: Cause },
    DrawPrizes { p: u8, prizes: u8, destination: ListRef },
    MoveCards { source: ListRef, destination: ListRef, cards: Option<List<120>>, count: Option<i32>, to_top: bool, to_bottom: bool, skip_cleanup: bool, source_card: CardId },
    EffectOfAbility { p: u8, power: PowerRef, card: CardId, target: Option<SlotRef>, cause: Cause },
    SpecialEnergy { p: u8, card: CardId, attached_to: SlotRef, exempt: bool },
    PlaceDamageCounters { p: u8, target: SlotRef, damage: i32, source: CardId, cause: Cause },
    MoveDamageCounters { p: u8 },
    MovedToActive { p: u8, card: CardId, cause: Cause },
    MovedFromActiveToBench { p: u8, card: CardId, cause: Cause },

    // ---- attack sub-effects ----
    ApplyWeakness { b: AtkBase, damage: i32, ignore_weakness: bool, ignore_resistance: bool },
    DealDamage { b: AtkBase, damage: i32 },
    PutDamage { b: AtkBase, damage: i32, weakness_applied: bool, survive_on_ten_hp: bool },
    AfterDamage { b: AtkBase, damage: i32 },
    /// `AttackTriggerEffect`: a step 7 trigger resolves (`target` is the damaged Pokémon, `source` the Attacking
    /// Pokémon). Only `card` reacts. Has no AtkBase: it is not an effect of the attack for Mist Energy.
    AttackTrigger {
        attack_effect: EffId,
        p: u8,
        opp: u8,
        attack: AttackRef,
        card: CardId,
        target: SlotRef,
        damage: i32,
        source: SlotRef,
        source_in_play: bool,
        retaliate: Option<crate::state::StoredRetaliate>,
    },
    PutCounters { b: AtkBase, damage: i32 },
    KnockOutOpponent { b: AtkBase, knocked_out: bool, prize_count: i32 },
    /// `KnockOutPlayerEffect` (KNOCK_OUT_PLAYERS_ACTIVE_POKEMON): the opponent takes the Prizes.
    KnockOutPlayer { b: AtkBase, knocked_out: bool, prize_count: i32 },
    DiscardCards { b: AtkBase, cards: SVec<CardId, 64> },
    CardsToHand { b: AtkBase, cards: SVec<CardId, 64> },
    GustOpponentBench { b: AtkBase },
    /// `MoveOpponentEnergyEffect`: `b.target` is the source slot.
    MoveOpponentEnergy { b: AtkBase, card: CardId, destination: SlotRef },
    AddMarker { b: AtkBase, marker: u16, marker_source: CardId },
    AddSpecialConditions { b: AtkBase, conditions: SVec<u8, 5>, poison_damage: Option<i32>, burn_damage: Option<i32>, confusion_damage: Option<i32> },
    RemoveSpecialConditions { b: AtkBase, conditions: SVec<u8, 5> },
    HealTarget { b: AtkBase, damage: i32 },
    /// `PlayLockEffect` (target = attacker's slot): the opponent gets the lock for their next turn.
    PlayLock { b: AtkBase, lock: &'static crate::spec::passive::LockDecl },
    /// `PreventRetreatEffect` (EffectOfAttackEffect): `opponent.active.cannotRetreatNextTurn = true`.
    PreventRetreat { b: AtkBase },
    /// `OpponentPokemonCannotUseAttackEffect` (EffectOfAttackEffect):
    /// `opponent.active.blockedAttackNameNextTurn = name`.
    OpponentPokemonCannotUseAttack { b: AtkBase, name: &'static str },
    /// `PreventAttackUntilLeavesActiveEffect` (EffectOfAttackEffect):
    /// `source.blockedAttackNameUntilLeavesActive = name` (source = target = the attacker's slot).
    PreventAttackUntilLeavesActive { b: AtkBase, name: &'static str },
    /// `DefendingPokemonTakesMoreDamageDuringAttackerNextTurnEffect`
    /// (EffectOfAttackEffect): arms `defendingPokemonExtraDamage*` on the
    /// opponent's current Active.
    DefendingPokemonTakesMoreDamage { b: AtkBase, damage_bonus: i32 },
    /// `AddSpecialConditionsPowerEffect` (check-effects; non-attack source).
    AddSpecialConditionsPower { p: u8, source: CardId, target: SlotRef, conditions: SVec<u8, 5>, poison_damage: i32, burn_damage: i32, sleep_flips: i32, confusion_damage: i32, cause: Cause },
    /// `ReduceDamageEffect` (EffectOfAttackEffect): the opponent's Active gets
    /// `attackDamageReductionNextTurn = max(0, reduction)`.
    ReduceDamage { b: AtkBase, reduction: i32 },
    /// `PreventDamageEffect` with non-empty `PreventDamageOptions` (same
    /// Twinleaf class/kind as [`Effect::PreventDamage`]).
    PreventDamageFiltered { b: AtkBase, filter: crate::state::PreventFilter },
    /// `SelfPreventRetreatEffect` (target = base.source, the attacker):
    /// `player.active.cannotRetreatNextTurnPending = true`.
    SelfPreventRetreat { b: AtkBase },
    /// `DiscardAttackerEnergyIfKnockedOutDuringOpponentsNextTurnEffect`
    /// (target = base.source; `markerSource` = `source_card`).
    DiscardAttackerEnergyIfKnockedOut { b: AtkBase, source_card: CardId },
    /// `SwitchOutOpponentsActiveEffect`: switches `bench_target` in when set.
    SwitchOutOpponentsActive { b: AtkBase, bench_target: Option<SlotRef> },
    /// `PreventDamageEffect` (EffectOfAttackEffect, target = attacker):
    /// `player.active.preventDamageNextTurnPending = {}` (empty filter only).
    PreventDamage { b: AtkBase },
    /// `PreventEffectsOfAttacksEffect` (EffectOfAttackEffect, target = attacker):
    /// `player.active.preventEffectsOfAttacksNextTurnPending = {}` (empty filter only).
    PreventEffectsOfAttacks { b: AtkBase },
    /// `ThisPokemonHasNoWeaknessDuringOpponentsNextTurnEffect` (target = attacker):
    /// `player.active.noWeaknessNextTurnPending = true`.
    ThisPokemonHasNoWeakness { b: AtkBase },
    /// `IncreaseDefendingPokemonAttackCostNextTurnEffect` (EffectOfAttackEffect):
    /// `opponent.active.attackCostIncreaseNextTurnPending = 1` (+ attacker id).
    IncreaseAttackCostNextTurn { b: AtkBase },
    /// `IncreaseDefendingPokemonRetreatCostNextTurnEffect` (EffectOfAttackEffect).
    IncreaseRetreatCostNextTurn { b: AtkBase },
    /// `CoinFlipCancelTrainerPlayEffect` (EffectOfAttackEffect, target = source):
    /// `opponent.coinFlipCancelTrainerPlayTurnsRemaining = max(.., 1)`.
    CoinFlipCancelTrainerPlay { b: AtkBase },
    /// `OpponentPokemonCannotAttackDuringTheirNextTurnEffect` (target = source):
    /// `max_energy` None locks all attacks, Some(n) only Pokémon with <= n Energy.
    OpponentPokemonCannotAttackNextTurn { b: AtkBase, max_energy: Option<i32> },
    /// `RetaliateOnDamageDuringOpponentsNextTurnEffect` (target = attacker,
    /// `{ damage }` options): `player.active.retaliateOnDamageNextTurnPending`.
    RetaliateOnDamage { b: AtkBase, damage: i32, source_card: CardId },
    /// `RetaliateDamageEffect`: `target.damage += damage` (b.player is the
    /// retaliator's owner, b.source its slot, b.target the attacker's slot).
    RetaliateDamage { b: AtkBase, damage: i32 },
    /// `MoveCountersAttackEffect`: moves `damage` of damage counters from
    /// `b.source` (the counters' source slot, which the TS constructor
    /// assigns over the attacker's) to `b.target`. Reducer-less: the card
    /// applies the counters after reducing it.
    MoveCounters { b: AtkBase, damage: i32 },
    /// `DevolveEffect`: the probe that asks whether devolving `b.target` as an effect of an attack (Espeon ex's
    /// Amethyst) is prevented (Mist Energy and the like). Reducer-less; the Devolve event follows when it isn't.
    DevolveProbe { b: AtkBase },

    // ---- play card ----
    AttachEnergy { p: u8, card: CardId, target: SlotRef, cause: Cause },
    PlaySupporter { p: u8, card: CardId, target: Option<SlotRef> },
    PlayStadium { p: u8, card: CardId },
    AttachPokemonTool { p: u8, card: CardId, target: SlotRef },
    PlayItem { p: u8, card: CardId, target: Option<SlotRef> },
    /// `via_attack` = `usedAsAttackEffect`: a Supporter's effect used as the effect of an attack
    /// (Mr. Mime's Look-Alike Show; rulings 1727, 1728, 1844, 1853).
    Trainer { p: u8, card: CardId, target: Option<SlotRef>, via_attack: bool },
    Energy { p: u8, card: CardId },
    Tool { p: u8, card: CardId },
    Stadium { p: u8, target: Option<SlotRef>, stadium: CardId, skip_ability_lock_check: bool },
    Supporter { p: u8, card: CardId },
    TrainerTarget { p: u8, card: CardId, target: Option<SlotRef> },
    DiscardToHand { p: u8, card: CardId },
    /// `mode`: 0 = until tails, n = n flips. `callback` indexes `coin_callbacks`.
    CoinFlipSequence { p: u8, mode: u8, callback: u8, skip_reflip_stadium: bool, skip_reflip_tool: bool },
    CoinFlip { p: u8, callback: Option<u8>, result: Option<bool>, skip_reflip_stadium: bool, skip_reflip_tool: bool },
}

impl Effect {
    /// Twinleaf `effect.type` string.
    pub fn type_name(&self) -> &'static str {
        use Effect::*;
        match self {
            BeginTurn { .. } => "BEGIN_TURN_EFFECT",
            DrawCardForTurn { .. } => "DRAW_CARD_FOR_TURN_EFFECT",
            DrewTopdeck { .. } => "DREW_TOPDECK_EFFECT",
            EndTurn { .. } => "END_TURN_EFFECT",
            WhoBegins { .. } => "END_TURN_EFFECT",
            BetweenTurns { .. } => "BETWEEN_TURNS_EFFECT",
            AfterAttack { .. } => "AFTER_ATTACK_EFFECT",
            AfterAttackTriggers { .. } => "AFTER_ATTACK_TRIGGERS_EFFECT",
            BeforeDoingDamage { .. } => "BEFORE_DOING_DAMAGE_EFFECT",
            CheckHp { .. } => "CHECK_HP_EFFECT",
            CheckPokemonStats { .. } => "CHECK_POKEMON_STATS_EFFECT",
            CheckPokemonType { .. } => "CHECK_POKEMON_TYPE_EFFECT",
            CheckRetreatCost { .. } => "CHECK_RETREAT_COST_EFFECT",
            CheckAttackCost { .. } => "CHECK_ATTACK_COST_EFFECT",
            CheckProvidedEnergy { .. } => "CHECK_ENOUGH_ENERGY_EFFECT",
            CheckPokemonPowers { .. } => "CHECK_POKEMON_POWERS_EFFECT",
            CheckPokemonAttacks { .. } => "CHECK_POKEMON_ATTACKS_EFFECT",
            CheckTableState { .. } => "CHECK_TABLE_STATE_EFFECT",
            CheckPrizesDestination { .. } => "CHECK_PRIZES_DESTINATION_EFFECT",
            CheckSpecialConditionRemoval { .. } => "CHECK_SPECIAL_CONDITION_REMOVAL_EFFECT",
            Retreat { .. } => "RETREAT_EFFECT",
            RetreatStart { .. } => "RETREAT_START_EFFECT",
            UseAttack { .. } => "USE_ATTACK_EFFECT",
            UseStadium { .. } => "USE_STADIUM_EFFECT",
            UsePower { .. } => "USE_POWER_EFFECT",
            Power { .. } => "POWER_EFFECT",
            Attack { .. } => "ATTACK_EFFECT",
            KnockOut { .. } => "KNOCK_OUT_EFFECT",
            Heal { .. } => "HEAL_EFFECT",
            Evolve { .. } => "EVOLVE_EFFECT",
            DrawPrizes { .. } => "DRAW_PRIZES_EFFECT",
            MoveCards { .. } => "MOVE_CARDS_EFFECT",
            EffectOfAbility { .. } => "EFFECT_OF_ABILITY_EFFECT",
            SpecialEnergy { .. } => "SPECIAL_ENERGY_EFFECT",
            PlaceDamageCounters { .. } => "PLACE_DAMAGE_COUNTERS_EFFECT",
            MovedToActive { .. } => "MOVED_TO_ACTIVE_EFFECT",
            MovedFromActiveToBench { .. } => "MOVED_FROM_ACTIVE_TO_BENCH_EFFECT",
            ApplyWeakness { .. } => "APPLY_WEAKNESS_EFFECT",
            DealDamage { .. } => "DEAL_DAMAGE_EFFECT",
            PutDamage { .. } => "PUT_DAMAGE_EFFECT",
            AfterDamage { .. } => "AFTER_DAMAGE_EFFECT",
            AttackTrigger { .. } => "ATTACK_TRIGGER_EFFECT",
            PutCounters { .. } => "PUT_COUNTERS_EFFECT",
            KnockOutOpponent { .. } => "KNOCK_OUT_OPPONENT_EFFECT",
            KnockOutPlayer { .. } => "KNOCK_OUT_PLAYER_EFFECT",
            DiscardCards { .. } => "DISCARD_CARD_EFFECT",
            CardsToHand { .. } => "CARDS_TO_HAND_EFFECT",
            GustOpponentBench { .. } => "GUST_OPPONENT_BENCH_EFFECT",
            MoveOpponentEnergy { .. } => "MOVE_OPPONENT_ENERGY_EFFECT",
            AddMarker { .. } => "ADD_MARKER_EFFECT",
            AddSpecialConditions { .. } => "ADD_SPECIAL_CONDITIONS_EFFECT",
            RemoveSpecialConditions { .. } => "REMOVE_SPECIAL_CONDITIONS_EFFECT",
            HealTarget { .. } => "HEAL_TARGET_EFFECT",
            PlayLock { .. } => "PLAY_LOCK_EFFECT",
            MoveDamageCounters { .. } => "MOVE_DAMAGE_COUNTERS_EFFECT",
            PreventRetreat { .. } => "PREVENT_RETREAT_EFFECT",
            OpponentPokemonCannotUseAttack { .. } => "OPPONENT_POKEMON_CANNOT_USE_ATTACK_EFFECT",
            PreventAttackUntilLeavesActive { .. } => "EFFECT_OF_ATTACK_EFFECT",
            DefendingPokemonTakesMoreDamage { .. } => "DEFENDING_POKEMON_TAKES_MORE_DAMAGE_DURING_ATTACKER_NEXT_TURN_EFFECT",
            AddSpecialConditionsPower { .. } => "ADD_SPECIAL_CONDITIONS_EFFECT",
            ReduceDamage { .. } => "REDUCE_DAMAGE_EFFECT",
            PreventDamageFiltered { .. } => "PREVENT_DAMAGE_EFFECT",
            SelfPreventRetreat { .. } => "SELF_PREVENT_RETREAT_EFFECT",
            DiscardAttackerEnergyIfKnockedOut { .. } => "DISCARD_ATTACKER_ENERGY_IF_KNOCKED_OUT_DURING_OPPONENTS_NEXT_TURN_EFFECT",
            SwitchOutOpponentsActive { .. } => "SWITCH_OUT_OPPONENTS_ACTIVE_EFFECT",
            PreventDamage { .. } => "PREVENT_DAMAGE_EFFECT",
            PreventEffectsOfAttacks { .. } => "PREVENT_EFFECTS_OF_ATTACKS_EFFECT",
            ThisPokemonHasNoWeakness { .. } => "THIS_POKEMON_HAS_NO_WEAKNESS_DURING_OPPONENTS_NEXT_TURN_EFFECT",
            IncreaseAttackCostNextTurn { .. } | IncreaseRetreatCostNextTurn { .. } => "EFFECT_OF_ATTACK_EFFECT",
            CoinFlipCancelTrainerPlay { .. } => "COIN_FLIP_CANCEL_TRAINER_PLAY_EFFECT",
            OpponentPokemonCannotAttackNextTurn { .. } => "OPPONENT_POKEMON_CANNOT_ATTACK_DURING_THEIR_NEXT_TURN_EFFECT",
            RetaliateOnDamage { .. } => "RETALIATE_ON_DAMAGE_DURING_OPPONENTS_NEXT_TURN_EFFECT",
            RetaliateDamage { .. } => "RETALIATE_DAMAGE_EFFECT",
            MoveCounters { .. } => "MOVE_COUNTERS_EFFECT",
            DevolveProbe { .. } => "DEVOLVE_EFFECT",
            Devolve { .. } => "DEVOLVE_EVENT",
            Swap { .. } => "SWAP_EVENT",
            AttachEnergy { .. } => "ATTACH_ENERGY_EFFECT",
            EnterPlay { .. } => "ENTER_PLAY_EVENT",
            PlaySupporter { .. } => "PLAY_SUPPORTER_EFFECT",
            PlayStadium { .. } => "PLAY_STADIUM_EFFECT",
            AttachPokemonTool { .. } => "PLAY_POKEMON_TOOL_EFFECT",
            PlayItem { .. } => "PLAY_ITEM_EFFECT",
            Trainer { .. } => "TRAINER_EFFECT",
            Energy { .. } => "ENERGY_EFFECT",
            Tool { .. } => "TOOL_EFFECT",
            Stadium { .. } => "STADIUM_EFFECT",
            Supporter { .. } => "SUPPORTER_EFFECT",
            TrainerTarget { .. } => "TRAINER_TARGET_EFFECT",
            DiscardToHand { .. } => "DISCARD_TO_HAND_EFFECT",
            CoinFlipSequence { .. } => "COIN_FLIP_SEQUENCE_EFFECT",
            CoinFlip { .. } => "COIN_FLIP_EFFECT",
        }
    }

    pub fn atk_base(&self) -> Option<&AtkBase> {
        use Effect::*;
        match self {
            ApplyWeakness { b, .. }
            | DealDamage { b, .. }
            | PutDamage { b, .. }
            | AfterDamage { b, .. }
            | PutCounters { b, .. }
            | KnockOutOpponent { b, .. }
            | KnockOutPlayer { b, .. }
            | DiscardCards { b, .. }
            | CardsToHand { b, .. }
            | GustOpponentBench { b, .. }
            | MoveOpponentEnergy { b, .. }
            | AddMarker { b, .. }
            | AddSpecialConditions { b, .. }
            | RemoveSpecialConditions { b, .. }
            | HealTarget { b, .. }
            | PlayLock { b, .. } => Some(b),
            | PreventRetreat { b } => Some(b),
            ReduceDamage { b, .. } | SwitchOutOpponentsActive { b, .. } => Some(b),
            PreventDamageFiltered { b, .. } | SelfPreventRetreat { b } | DiscardAttackerEnergyIfKnockedOut { b, .. } => Some(b),
            OpponentPokemonCannotUseAttack { b, .. } | PreventAttackUntilLeavesActive { b, .. } => Some(b),
            DefendingPokemonTakesMoreDamage { b, .. } => Some(b),
            PreventDamage { b } | PreventEffectsOfAttacks { b } => Some(b),
            ThisPokemonHasNoWeakness { b } => Some(b),
            IncreaseAttackCostNextTurn { b } | IncreaseRetreatCostNextTurn { b } | CoinFlipCancelTrainerPlay { b } => Some(b),
            OpponentPokemonCannotAttackNextTurn { b, .. } => Some(b),
            RetaliateOnDamage { b, .. } | RetaliateDamage { b, .. } | MoveCounters { b, .. } | DevolveProbe { b } => Some(b),
            _ => None,
        }
    }

    pub fn atk_base_mut(&mut self) -> Option<&mut AtkBase> {
        use Effect::*;
        match self {
            ApplyWeakness { b, .. }
            | DealDamage { b, .. }
            | PutDamage { b, .. }
            | AfterDamage { b, .. }
            | PutCounters { b, .. }
            | KnockOutOpponent { b, .. }
            | KnockOutPlayer { b, .. }
            | DiscardCards { b, .. }
            | CardsToHand { b, .. }
            | GustOpponentBench { b, .. }
            | MoveOpponentEnergy { b, .. }
            | AddMarker { b, .. }
            | AddSpecialConditions { b, .. }
            | RemoveSpecialConditions { b, .. }
            | HealTarget { b, .. }
            | PlayLock { b, .. } => Some(b),
            | PreventRetreat { b } => Some(b),
            ReduceDamage { b, .. } | SwitchOutOpponentsActive { b, .. } => Some(b),
            PreventDamageFiltered { b, .. } | SelfPreventRetreat { b } | DiscardAttackerEnergyIfKnockedOut { b, .. } => Some(b),
            OpponentPokemonCannotUseAttack { b, .. } | PreventAttackUntilLeavesActive { b, .. } => Some(b),
            DefendingPokemonTakesMoreDamage { b, .. } => Some(b),
            PreventDamage { b } | PreventEffectsOfAttacks { b } => Some(b),
            ThisPokemonHasNoWeakness { b } => Some(b),
            IncreaseAttackCostNextTurn { b } | IncreaseRetreatCostNextTurn { b } | CoinFlipCancelTrainerPlay { b } => Some(b),
            OpponentPokemonCannotAttackNextTurn { b, .. } => Some(b),
            RetaliateOnDamage { b, .. } | RetaliateDamage { b, .. } | MoveCounters { b, .. } | DevolveProbe { b } => Some(b),
            _ => None,
        }
    }

    /// Subscription class used to skip cards that never react to an effect.
    pub fn kind(&self) -> u32 {
        use Effect::*;
        let k = match self {
            BeginTurn { .. } => 0,
            DrawCardForTurn { .. } => 1,
            DrewTopdeck { .. } => 2,
            EndTurn { .. } => 3,
            WhoBegins { .. } => 4,
            BetweenTurns { .. } => 5,
            AfterAttack { .. } => 6,
            AfterAttackTriggers { .. } => 245,
            BeforeDoingDamage { .. } => 7,
            CheckHp { .. } => 8,
            CheckPokemonStats { .. } => 9,
            CheckPokemonType { .. } => 10,
            CheckRetreatCost { .. } => 11,
            CheckAttackCost { .. } => 12,
            CheckProvidedEnergy { .. } => 13,
            CheckPokemonPowers { .. } => 14,
            CheckPokemonAttacks { .. } => 15,
            CheckTableState { .. } => 17,
            CheckPrizesDestination { .. } => 18,
            CheckSpecialConditionRemoval { .. } => 19,
            Retreat { .. } => 20,
            RetreatStart { .. } => 21,
            UseAttack { .. } => 22,
            UseStadium { .. } => 23,
            UsePower { .. } => 24,
            Power { .. } => 25,
            Attack { .. } => 26,
            KnockOut { .. } => 27,
            Heal { .. } => 28,
            Evolve { .. } => 29,
            DrawPrizes { .. } => 30,
            MoveCards { .. } => 31,
            EffectOfAbility { .. } => 32,
            SpecialEnergy { .. } => 33,
            PlaceDamageCounters { .. } => 34,
            MovedToActive { .. } => 35,
            MovedFromActiveToBench { .. } => 36,
            ApplyWeakness { .. } => 37,
            DealDamage { .. } => 38,
            PutDamage { .. } => 39,
            AfterDamage { .. } => 40,
            AttackTrigger { .. } => 246,
            PutCounters { .. } => 41,
            KnockOutOpponent { .. } => 42,
            KnockOutPlayer { .. } => 140,
            DiscardCards { .. } => 43,
            CardsToHand { .. } => 44,
            GustOpponentBench { .. } => 45,
            MoveOpponentEnergy { .. } => 164,
            AddMarker { .. } => 46,
            AddSpecialConditions { .. } => 47,
            RemoveSpecialConditions { .. } => 48,
            HealTarget { .. } => 49,
            AttachEnergy { .. } => 50,
            EnterPlay { .. } => 51,
            PlaySupporter { .. } => 52,
            PlayStadium { .. } => 53,
            AttachPokemonTool { .. } => 54,
            PlayItem { .. } => 55,
            Trainer { .. } => 56,
            Energy { .. } => 57,
            Tool { .. } => 58,
            Stadium { .. } => 59,
            Supporter { .. } => 60,
            CoinFlip { .. } => 61,
            TrainerTarget { .. } => 62,
            DiscardToHand { .. } => 63,
            CoinFlipSequence { .. } => 66,
            PlayLock { .. } => 67,
            MoveDamageCounters { .. } => 68,
            PreventRetreat { .. } => 69,
            AddSpecialConditionsPower { .. } => 70,
            ReduceDamage { .. } => 110,
            PreventDamageFiltered { .. } => 84,
            SelfPreventRetreat { .. } => 105,
            DiscardAttackerEnergyIfKnockedOut { .. } => 106,
            SwitchOutOpponentsActive { .. } => 111,
            PreventDamage { .. } => 84,
            PreventEffectsOfAttacks { .. } => 77,
            ThisPokemonHasNoWeakness { .. } => 148,
            RetaliateOnDamage { .. } => 172,
            RetaliateDamage { .. } => 173,
            MoveCounters { .. } => 244,
            DevolveProbe { .. } => 247,
            Devolve { .. } => 248,
            Swap { .. } => 249,
            OpponentPokemonCannotUseAttack { .. } => 91,
            PreventAttackUntilLeavesActive { .. } => 188,
            DefendingPokemonTakesMoreDamage { .. } => 130,
            IncreaseAttackCostNextTurn { .. } => 120,
            IncreaseRetreatCostNextTurn { .. } => 121,
            CoinFlipCancelTrainerPlay { .. } => 122,
            OpponentPokemonCannotAttackNextTurn { .. } => 196,
        };
        k
    }
}

/// `Effect::kind()` values, for subscription masks.
pub mod k {
    pub const BEGIN_TURN: u32 = 0;
    pub const DRAW_CARD_FOR_TURN: u32 = 1;
    pub const DREW_TOPDECK: u32 = 2;
    pub const END_TURN: u32 = 3;
    pub const WHO_BEGINS: u32 = 4;
    pub const BETWEEN_TURNS: u32 = 5;
    pub const AFTER_ATTACK: u32 = 6;
    pub const AFTER_ATTACK_TRIGGERS: u32 = 245;
    pub const ATTACK_TRIGGER: u32 = 246;
    pub const BEFORE_DOING_DAMAGE: u32 = 7;
    pub const CHECK_HP: u32 = 8;
    pub const CHECK_POKEMON_STATS: u32 = 9;
    pub const CHECK_POKEMON_TYPE: u32 = 10;
    pub const CHECK_RETREAT_COST: u32 = 11;
    pub const CHECK_ATTACK_COST: u32 = 12;
    pub const CHECK_PROVIDED_ENERGY: u32 = 13;
    pub const CHECK_POKEMON_POWERS: u32 = 14;
    pub const CHECK_POKEMON_ATTACKS: u32 = 15;
    pub const CHECK_TABLE_STATE: u32 = 17;
    pub const CHECK_PRIZES_DESTINATION: u32 = 18;
    pub const CHECK_SPECIAL_CONDITION_REMOVAL: u32 = 19;
    pub const RETREAT: u32 = 20;
    pub const RETREAT_START: u32 = 21;
    pub const USE_ATTACK: u32 = 22;
    pub const USE_STADIUM: u32 = 23;
    pub const USE_POWER: u32 = 24;
    pub const POWER: u32 = 25;
    pub const ATTACK: u32 = 26;
    pub const KNOCK_OUT: u32 = 27;
    pub const HEAL: u32 = 28;
    pub const EVOLVE: u32 = 29;
    pub const DRAW_PRIZES: u32 = 30;
    pub const MOVE_CARDS: u32 = 31;
    pub const EFFECT_OF_ABILITY: u32 = 32;
    pub const SPECIAL_ENERGY: u32 = 33;
    pub const PLACE_DAMAGE_COUNTERS: u32 = 34;
    pub const MOVED_TO_ACTIVE: u32 = 35;
    pub const MOVED_FROM_ACTIVE_TO_BENCH: u32 = 36;
    pub const APPLY_WEAKNESS: u32 = 37;
    pub const DEAL_DAMAGE: u32 = 38;
    pub const PUT_DAMAGE: u32 = 39;
    pub const AFTER_DAMAGE: u32 = 40;
    pub const MOVE_OPPONENT_ENERGY: u32 = 164;
    pub const PUT_COUNTERS: u32 = 41;
    pub const KNOCK_OUT_OPPONENT: u32 = 42;
    pub const KNOCK_OUT_PLAYER: u32 = 140;
    pub const DISCARD_CARDS: u32 = 43;
    pub const CARDS_TO_HAND: u32 = 44;
    pub const GUST_OPPONENT_BENCH: u32 = 45;
    pub const ADD_MARKER: u32 = 46;
    pub const ADD_SPECIAL_CONDITIONS: u32 = 47;
    pub const REMOVE_SPECIAL_CONDITIONS: u32 = 48;
    pub const HEAL_TARGET: u32 = 49;
    pub const ATTACH_ENERGY: u32 = 50;
    pub const ENTER_PLAY: u32 = 51;
    pub const PLAY_SUPPORTER: u32 = 52;
    pub const PLAY_STADIUM: u32 = 53;
    pub const ATTACH_POKEMON_TOOL: u32 = 54;
    pub const PLAY_ITEM: u32 = 55;
    pub const TRAINER: u32 = 56;
    pub const ENERGY: u32 = 57;
    pub const TOOL: u32 = 58;
    pub const STADIUM: u32 = 59;
    pub const SUPPORTER: u32 = 60;
    pub const COIN_FLIP: u32 = 61;
    pub const TRAINER_TARGET: u32 = 62;
    pub const DISCARD_TO_HAND: u32 = 63;
    pub const COIN_FLIP_SEQUENCE: u32 = 66;
    pub const PLAY_LOCK: u32 = 67;
    pub const MOVE_DAMAGE_COUNTERS: u32 = 68;
    pub const PREVENT_RETREAT: u32 = 69;
    pub const ADD_SPECIAL_CONDITIONS_POWER: u32 = 70;
    pub const REDUCE_DAMAGE: u32 = 110;
    pub const SELF_PREVENT_RETREAT: u32 = 105;
    pub const DISCARD_ATTACKER_ENERGY_IF_KO: u32 = 106;
    pub const SWITCH_OUT_OPPONENTS_ACTIVE: u32 = 111;
    pub const THIS_POKEMON_HAS_NO_WEAKNESS: u32 = 148;
    pub const RETALIATE_ON_DAMAGE: u32 = 172;
    pub const RETALIATE_DAMAGE: u32 = 173;
    pub const MOVE_COUNTERS: u32 = 244;
    pub const DEVOLVE_PROBE: u32 = 247;
    pub const DEVOLVE: u32 = 248;
    pub const SWAP: u32 = 249;
    // Declaration markers (never dispatched): set in a card's mask when it declares a permission, a
    // restriction or rule limits (`CardSpec::restricts` / `limits`) or a lock over events, so `Game::kinds_present` says whether a game has any.
    pub const DECLARES_PERMIT: u32 = 250;
    pub const DECLARES_RESTRICT: u32 = 251;
    pub const DECLARES_EVENT_LOCK: u32 = 252;
    /// A permission that lifts `Limit::FirstTurn` / `BaseEnteredThisTurn` / `EvolvesFrom` (with `DECLARES_PERMIT`).
    pub const PERMIT_FIRST_TURN: u32 = 253;
    pub const PERMIT_BASE_ENTERED: u32 = 254;
    pub const PERMIT_EVOLVES_FROM: u32 = 255;
    pub const PREVENT_DAMAGE: u32 = 84;
    pub const PREVENT_EFFECTS_OF_ATTACKS: u32 = 77;
    pub const OPPONENT_POKEMON_CANNOT_USE_ATTACK: u32 = 91;
    pub const PREVENT_ATTACK_UNTIL_LEAVES_ACTIVE: u32 = 188;
    pub const DEFENDING_POKEMON_TAKES_MORE_DAMAGE: u32 = 130;
    pub const INCREASE_ATTACK_COST_NEXT_TURN: u32 = 120;
    pub const INCREASE_RETREAT_COST_NEXT_TURN: u32 = 121;
    pub const COIN_FLIP_CANCEL_TRAINER_PLAY: u32 = 122;
    pub const OPPONENT_POKEMON_CANNOT_ATTACK_NEXT_TURN: u32 = 196;
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
            MovedToActive { cause, .. }
            | MovedFromActiveToBench { cause, .. }
            | Heal { cause, .. }
            | Evolve { cause, .. }
            | EffectOfAbility { cause, .. }
            | PlaceDamageCounters { cause, .. }
            | AddSpecialConditionsPower { cause, .. }
            | AttachEnergy { cause, .. }
            | EnterPlay { cause, .. }
            | Devolve { cause, .. }
            | Swap { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
