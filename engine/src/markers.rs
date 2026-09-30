//! Marker names as small integers. Twinleaf markers are strings; every name
//! the engine or a card can place is registered here once.

pub type MarkerName = u16;

macro_rules! markers {
    ($($id:ident = $s:expr),* $(,)?) => {
        pub const MARKER_NAMES: &[&str] = &[$($s),*];
        #[allow(non_camel_case_types, dead_code)]
        #[repr(u16)]
        enum MarkerIdx { $($id),* }
        $(pub const $id: MarkerName = MarkerIdx::$id as u16;)*
    };
}

markers! {
    DAMAGE_DEALT_MARKER = "DAMAGE_DEALT_MARKER",
    CLEAR_KNOCKOUT_MARKER = "CLEAR_KNOCKOUT_MARKER",
    KNOCKOUT_MARKER = "KNOCKOUT_MARKER",
    COIN_REFLIP_AGAIN_USED = "COIN_REFLIP_AGAIN_USED",
    LOST_CITY_MARKER = "LOST_CITY_MARKER",
}

pub fn marker_name(id: MarkerName) -> &'static str {
    MARKER_NAMES[id as usize]
}

pub fn marker_id(name: &str) -> Option<MarkerName> {
    MARKER_NAMES.iter().position(|n| *n == name).map(|i| i as MarkerName)
}

/// `MarkerSourceType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum SourceType {
    #[default]
    None = 0,
    Attack = 1,
    Ability = 2,
    Trainer = 3,
    Energy = 4,
    Stadium = 5,
}

impl SourceType {
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            SourceType::None => None,
            SourceType::Attack => Some("attack"),
            SourceType::Ability => Some("ability"),
            SourceType::Trainer => Some("trainer"),
            SourceType::Energy => Some("energy"),
            SourceType::Stadium => Some("stadium"),
        }
    }
}

/// `MarkerTargetScope`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum TargetScope {
    #[default]
    None = 0,
    Pokemon = 1,
    Player = 2,
}

impl TargetScope {
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            TargetScope::None => None,
            TargetScope::Pokemon => Some("pokemon"),
            TargetScope::Player => Some("player"),
        }
    }
}
