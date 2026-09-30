//! Effects: one variant per Twinleaf effect class (same names), carrying
//! indices instead of object references. Effects live in the store's arena
//! and are addressed by [`EffId`] so continuations can share them the way
//! Twinleaf closures share effect objects.

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

pub type EnergyMap = SVec<EnergyEntry, 40>;
pub type Cost = SVec<CardType, 10>;

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
    AfterAttack { p: u8, opp: u8, attack: AttackRef },
    BeforeDoingDamage { attack_effect: EffId, p: u8, opp: u8, attack: AttackRef },

    // ---- checks ----
    CheckHp { p: u8, target: SlotRef, card: Option<CardId> },
    CheckPokemonStats { target: SlotRef, weakness: SVec<WeaknessV, 4>, resistance: SVec<ResistanceV, 4> },
    CheckPokemonType { target: SlotRef, card_types: SVec<CardType, 4> },
    CheckRetreatCost { p: u8, cost: Cost },
    CheckAttackCost { p: u8, attack: AttackRef, cost: Cost },
    CheckProvidedEnergy { p: u8, source: SlotRef, energy_map: EnergyMap },
    CheckPokemonPowers { p: u8, target: CardId, powers: SVec<PowerRef, 8> },
    CheckPokemonAttacks { p: u8, attacks: SVec<AttackRef, 8> },
    CheckPokemonPlayedTurn { p: u8, target: SlotRef, pokemon_played_turn: i32 },
    CheckTableState { bench_sizes: [u8; 2] },
    CheckPrizesDestination { p: u8, destination: ListRef },
    CheckSpecialConditionRemoval { p: u8, target: SlotRef, preserved: SVec<u8, 5> },

    // ---- game ----
    Retreat { p: u8, bench_index: u8, ignore_status_conditions: bool, move_retreat_cost_to: ListRef },
    RetreatStart { p: u8 },
    UseAttack { p: u8, attack: AttackRef, source: SlotRef, ignore_status_conditions: bool, barrage_used: bool },
    UseStadium { p: u8, stadium: CardId },
    UsePower { p: u8, power: PowerRef, card: CardId, target: CardTarget, bench_target: Option<SlotRef> },
    /// `probe`: the lock-check stand-in power ('test') carrying the real power's flags.
    Power { p: u8, power: PowerRef, card: CardId, target: Option<SlotRef>, probe: bool },
    Attack { p: u8, opp: u8, attack: AttackRef, damage: i32, ignore_weakness: bool, ignore_resistance: bool, source: SlotRef, barrage_used: bool },
    KnockOut { p: u8, target: SlotRef, prize_count: i32, prize_destination: Option<ListRef>, attack: Option<AttackRef> },
    Heal { p: u8, target: SlotRef, damage: i32 },
    Evolve { p: u8, target: SlotRef, card: CardId },
    DrawPrizes { p: u8, prizes: u8, destination: ListRef },
    MoveCards { source: ListRef, destination: ListRef, cards: Option<SVec<CardId, 16>>, count: Option<i32>, to_top: bool, to_bottom: bool, skip_cleanup: bool, source_card: CardId },
    EffectOfAbility { p: u8, power: PowerRef, card: CardId, target: Option<SlotRef> },
    SpecialEnergy { p: u8, card: CardId, attached_to: SlotRef, exempt: bool },
    PlaceDamageCounters { p: u8, target: SlotRef, damage: i32, source: CardId },
    MovedToActive { p: u8, card: CardId },
    MovedFromActiveToBench { p: u8, card: CardId },

    // ---- attack sub-effects ----
    ApplyWeakness { b: AtkBase, damage: i32, ignore_weakness: bool, ignore_resistance: bool },
    DealDamage { b: AtkBase, damage: i32 },
    PutDamage { b: AtkBase, damage: i32, weakness_applied: bool, survive_on_ten_hp: bool },
    AfterDamage { b: AtkBase, damage: i32 },
    PutCounters { b: AtkBase, damage: i32 },
    KnockOutOpponent { b: AtkBase, knocked_out: bool, prize_count: i32 },
    DiscardCards { b: AtkBase, cards: SVec<CardId, 16> },
    CardsToHand { b: AtkBase, cards: SVec<CardId, 16> },
    GustOpponentBench { b: AtkBase },
    AddMarker { b: AtkBase, marker: u16, marker_source: CardId },
    AddSpecialConditions { b: AtkBase, conditions: SVec<u8, 5>, poison_damage: Option<i32>, burn_damage: Option<i32>, confusion_damage: Option<i32> },
    RemoveSpecialConditions { b: AtkBase, conditions: SVec<u8, 5> },
    HealTarget { b: AtkBase, damage: i32 },

    // ---- play card ----
    AttachEnergy { p: u8, card: CardId, target: SlotRef },
    PlayPokemon { p: u8, card: CardId, target: SlotRef, slot: SlotType, index: u8 },
    PlaySupporter { p: u8, card: CardId, target: Option<SlotRef> },
    PlayStadium { p: u8, card: CardId },
    AttachPokemonTool { p: u8, card: CardId, target: SlotRef },
    PlayItem { p: u8, card: CardId, target: Option<SlotRef> },
    Trainer { p: u8, card: CardId, target: Option<SlotRef> },
    Energy { p: u8, card: CardId },
    Tool { p: u8, card: CardId },
    Stadium { p: u8, target: Option<SlotRef>, stadium: CardId },
    Supporter { p: u8, card: CardId },
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
            BeforeDoingDamage { .. } => "BEFORE_DOING_DAMAGE_EFFECT",
            CheckHp { .. } => "CHECK_HP_EFFECT",
            CheckPokemonStats { .. } => "CHECK_POKEMON_STATS_EFFECT",
            CheckPokemonType { .. } => "CHECK_POKEMON_TYPE_EFFECT",
            CheckRetreatCost { .. } => "CHECK_RETREAT_COST_EFFECT",
            CheckAttackCost { .. } => "CHECK_ATTACK_COST_EFFECT",
            CheckProvidedEnergy { .. } => "CHECK_ENOUGH_ENERGY_EFFECT",
            CheckPokemonPowers { .. } => "CHECK_POKEMON_POWERS_EFFECT",
            CheckPokemonAttacks { .. } => "CHECK_POKEMON_ATTACKS_EFFECT",
            CheckPokemonPlayedTurn { .. } => "CHECK_POKEMON_PLAYED_TURN_EFFECT",
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
            PutCounters { .. } => "PUT_COUNTERS_EFFECT",
            KnockOutOpponent { .. } => "KNOCK_OUT_OPPONENT_EFFECT",
            DiscardCards { .. } => "DISCARD_CARD_EFFECT",
            CardsToHand { .. } => "CARDS_TO_HAND_EFFECT",
            GustOpponentBench { .. } => "GUST_OPPONENT_BENCH_EFFECT",
            AddMarker { .. } => "ADD_MARKER_EFFECT",
            AddSpecialConditions { .. } => "ADD_SPECIAL_CONDITIONS_EFFECT",
            RemoveSpecialConditions { .. } => "REMOVE_SPECIAL_CONDITIONS_EFFECT",
            HealTarget { .. } => "HEAL_TARGET_EFFECT",
            AttachEnergy { .. } => "ATTACH_ENERGY_EFFECT",
            PlayPokemon { .. } => "PLAY_POKEMON_EFFECT",
            PlaySupporter { .. } => "PLAY_SUPPORTER_EFFECT",
            PlayStadium { .. } => "PLAY_STADIUM_EFFECT",
            AttachPokemonTool { .. } => "PLAY_POKEMON_TOOL_EFFECT",
            PlayItem { .. } => "PLAY_ITEM_EFFECT",
            Trainer { .. } => "TRAINER_EFFECT",
            Energy { .. } => "ENERGY_EFFECT",
            Tool { .. } => "TOOL_EFFECT",
            Stadium { .. } => "STADIUM_EFFECT",
            Supporter { .. } => "SUPPORTER_EFFECT",
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
            | DiscardCards { b, .. }
            | CardsToHand { b, .. }
            | GustOpponentBench { b, .. }
            | AddMarker { b, .. }
            | AddSpecialConditions { b, .. }
            | RemoveSpecialConditions { b, .. }
            | HealTarget { b, .. } => Some(b),
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
            | DiscardCards { b, .. }
            | CardsToHand { b, .. }
            | GustOpponentBench { b, .. }
            | AddMarker { b, .. }
            | AddSpecialConditions { b, .. }
            | RemoveSpecialConditions { b, .. }
            | HealTarget { b, .. } => Some(b),
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
            BeforeDoingDamage { .. } => 7,
            CheckHp { .. } => 8,
            CheckPokemonStats { .. } => 9,
            CheckPokemonType { .. } => 10,
            CheckRetreatCost { .. } => 11,
            CheckAttackCost { .. } => 12,
            CheckProvidedEnergy { .. } => 13,
            CheckPokemonPowers { .. } => 14,
            CheckPokemonAttacks { .. } => 15,
            CheckPokemonPlayedTurn { .. } => 16,
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
            PutCounters { .. } => 41,
            KnockOutOpponent { .. } => 42,
            DiscardCards { .. } => 43,
            CardsToHand { .. } => 44,
            GustOpponentBench { .. } => 45,
            AddMarker { .. } => 46,
            AddSpecialConditions { .. } => 47,
            RemoveSpecialConditions { .. } => 48,
            HealTarget { .. } => 49,
            AttachEnergy { .. } => 50,
            PlayPokemon { .. } => 51,
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
        };
        k
    }
}

/// Bitmask over [`Effect::kind`].
pub type KindMask = u64;
pub const ALL_KINDS: KindMask = u64::MAX;
