//! Rules enums, numbered exactly as in Twinleaf (`card-types.ts`,
//! `pokemon-types.ts`, `state.ts`, `play-card-action.ts`) so canonical output
//! matches byte for byte.

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SuperType {
    None = 0,
    Pokemon = 1,
    Trainer = 2,
    Energy = 3,
    Any = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EnergyType {
    Basic = 0,
    Special = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TrainerType {
    Item = 0,
    Supporter = 1,
    Stadium = 2,
    Tool = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(u8)]
pub enum Stage {
    None = 0,
    Restored = 1,
    Basic = 2,
    Stage1 = 3,
    Stage2 = 4,
    Vmax = 5,
    Vstar = 6,
    Vunion = 7,
    Legend = 8,
    Mega = 9,
    Break = 10,
    LvX = 11,
}

impl Stage {
    pub fn from_u8(v: u8) -> Stage {
        use Stage::*;
        match v {
            0 => None,
            1 => Restored,
            2 => Basic,
            3 => Stage1,
            4 => Stage2,
            5 => Vmax,
            6 => Vstar,
            7 => Vunion,
            8 => Legend,
            9 => Mega,
            10 => Break,
            _ => LvX,
        }
    }
}

/// Energy / Pokémon types. Values above `NONE` are archetype markers, kept
/// as raw numbers.
pub type CardType = u8;
pub mod ct {
    use super::CardType;
    pub const ANY: CardType = 0;
    pub const GRASS: CardType = 1;
    pub const FIRE: CardType = 2;
    pub const WATER: CardType = 3;
    pub const LIGHTNING: CardType = 4;
    pub const PSYCHIC: CardType = 5;
    pub const FIGHTING: CardType = 6;
    pub const DARK: CardType = 7;
    pub const METAL: CardType = 8;
    pub const COLORLESS: CardType = 9;
    pub const FAIRY: CardType = 10;
    pub const DRAGON: CardType = 11;
    pub const NONE: CardType = 12;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(u8)]
pub enum SpecialCondition {
    Paralyzed = 0,
    Confused = 1,
    Asleep = 2,
    Poisoned = 3,
    Burned = 4,
}

impl SpecialCondition {
    pub fn from_u8(v: u8) -> SpecialCondition {
        match v {
            0 => SpecialCondition::Paralyzed,
            1 => SpecialCondition::Confused,
            2 => SpecialCondition::Asleep,
            3 => SpecialCondition::Poisoned,
            _ => SpecialCondition::Burned,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BoardEffect {
    AbilityUsed = 0,
    PowerGlow = 1,
    PowerNegatedGlow = 2,
    PowerReturn = 3,
    Evolve = 4,
    RevealOpponentHand = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PowerType {
    PokeBody = 0,
    PokePower = 1,
    Ability = 2,
    AncientTrait = 3,
    BabyRule = 4,
    HeldItem = 5,
    PokemonPower = 6,
    VunionAssembly = 7,
    LegendAssembly = 8,
    TrainerAbility = 9,
    HolonsSpecialEnergyEffect = 10,
    MegaEvolutionRule = 11,
    LvXRule = 12,
    BreakRule = 13,
    ArceusRule = 14,
    EnergyAbility = 15,
}

impl PowerType {
    pub fn from_u8(v: u8) -> PowerType {
        use PowerType::*;
        match v {
            0 => PokeBody,
            1 => PokePower,
            2 => Ability,
            3 => AncientTrait,
            4 => BabyRule,
            5 => HeldItem,
            6 => PokemonPower,
            7 => VunionAssembly,
            8 => LegendAssembly,
            9 => TrainerAbility,
            10 => HolonsSpecialEnergyEffect,
            11 => MegaEvolutionRule,
            12 => LvXRule,
            13 => BreakRule,
            14 => ArceusRule,
            _ => EnergyAbility,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(u8)]
pub enum GamePhase {
    WaitingForPlayers = 0,
    Setup = 1,
    PlayerTurn = 2,
    Attack = 3,
    AfterAttack = 4,
    ChoosePrizes = 5,
    BetweenTurns = 6,
    Finished = 7,
    Draw = 8,
}

/// `GameWinner`: NONE = -1, PLAYER_1 = 0, PLAYER_2 = 1, DRAW = 3.
pub type Winner = i8;
pub const WINNER_NONE: Winner = -1;
pub const WINNER_P1: Winner = 0;
pub const WINNER_P2: Winner = 1;
pub const WINNER_DRAW: Winner = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PlayerType {
    Any = 0,
    TopPlayer = 1,
    BottomPlayer = 2,
}

impl PlayerType {
    pub fn from_u8(v: u8) -> PlayerType {
        match v {
            0 => PlayerType::Any,
            1 => PlayerType::TopPlayer,
            _ => PlayerType::BottomPlayer,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SlotType {
    Board = 0,
    Active = 1,
    Bench = 2,
    Hand = 3,
    Discard = 4,
    LostZone = 5,
    Deck = 6,
}

impl SlotType {
    pub fn from_u8(v: u8) -> SlotType {
        use SlotType::*;
        match v {
            0 => Board,
            1 => Active,
            2 => Bench,
            3 => Hand,
            4 => Discard,
            5 => LostZone,
            _ => Deck,
        }
    }
}

/// `CardTarget` as sent by clients: relative player, slot kind, index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CardTarget {
    pub player: PlayerType,
    pub slot: SlotType,
    pub index: u8,
}

impl CardTarget {
    pub const fn new(player: PlayerType, slot: SlotType, index: u8) -> CardTarget {
        CardTarget { player, slot, index }
    }
}

/// Card tags as a bitset; bit positions follow [`TAG_NAMES`].
pub type Tags = u128;

pub const TAG_NAMES: &[&str] = &[
    "SP", "EX", "GX", "Lv.X", "V", "VMAX", "VSTAR", "V-UNION", "Ace Spec", "Radiant",
    "Secret Machine", "Technical Machine", "Team Plasma", "Fusion Strike", "Single Strike",
    "Rapid Strike", "ex", "Future", "Ancient", "Tera", "Ultra Beast", "Tag Team",
    "Team Aqua's", "Team Magma's", "Prism Star", "Star", "Baby", "Break", "Prime", "Holo",
    "Legend", "Dual Legend", "Dual Stadium", "Team Flare", "Mega EX", "Play During Setup",
    "Delta Species", "Dark", "Brock's", "Blaine's", "Lillie's", "N's", "Iono's", "Hop's",
    "Marnie's", "Steven's", "Ethan's", "Misty's", "Erika's", "Larry's", "Cynthia's",
    "Arven's", "Aura's", "Mega ex", "Holon's", "Rocket's", "Team Rocket's", "Unown",
    "Primal", "Arceus", "VS", "Antique",
];

pub mod tag {
    pub const POKEMON_SP: u32 = 0;
    pub const POKEMON_EX: u32 = 1;
    pub const POKEMON_GX: u32 = 2;
    pub const POKEMON_LV_X: u32 = 3;
    pub const POKEMON_V: u32 = 4;
    pub const POKEMON_VMAX: u32 = 5;
    pub const POKEMON_VSTAR: u32 = 6;
    pub const POKEMON_VUNION: u32 = 7;
    pub const ACE_SPEC: u32 = 8;
    pub const RADIANT: u32 = 9;
    pub const FUSION_STRIKE: u32 = 13;
    pub const SINGLE_STRIKE: u32 = 14;
    pub const RAPID_STRIKE: u32 = 15;
    pub const POKEMON_EX_LOWER: u32 = 16;
    pub const FUTURE: u32 = 17;
    pub const ANCIENT: u32 = 18;
    pub const POKEMON_TERA: u32 = 19;
    pub const TAG_TEAM: u32 = 21;
    pub const PRISM_STAR: u32 = 24;
    pub const BREAK: u32 = 27;
    pub const DUAL_LEGEND: u32 = 31;
    pub const DUAL_STADIUM: u32 = 32;
    pub const PLAY_DURING_SETUP: u32 = 35;
    pub const LILLIES: u32 = 40;
    pub const NS: u32 = 41;
    pub const IONOS: u32 = 42;
    pub const HOPS: u32 = 43;
    pub const MARNIES: u32 = 44;
    pub const STEVENS: u32 = 45;
    pub const ETHANS: u32 = 46;
    pub const CYNTHIAS: u32 = 50;
    pub const POKEMON_SV_MEGA: u32 = 53;
    pub const TEAM_ROCKET: u32 = 56;
    pub const ANTIQUE: u32 = 61;
}

pub fn tag_bit(t: u32) -> Tags {
    1u128 << t
}

pub fn tag_index(name: &str) -> Option<u32> {
    TAG_NAMES.iter().position(|n| *n == name).map(|i| i as u32)
}
