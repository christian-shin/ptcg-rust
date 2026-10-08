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

static EXTRA: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

thread_local! {
    /// Per-thread copies, so lookups don't contend on `EXTRA`: interned ids by
    /// the name's address and length, and the interned names by id.
    static IDS: std::cell::RefCell<Vec<(usize, usize, MarkerName)>> = const { std::cell::RefCell::new(Vec::new()) };
    static NAMES: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Name of a marker id (built-in or interned).
pub fn marker_name(id: MarkerName) -> &'static str {
    let i = id as usize;
    if i < MARKER_NAMES.len() {
        return MARKER_NAMES[i];
    }
    let j = i - MARKER_NAMES.len();
    NAMES.with(|n| {
        let mut n = n.borrow_mut();
        if j >= n.len() {
            *n = EXTRA.lock().unwrap().clone();
        }
        n[j]
    })
}

pub fn marker_id(name: &str) -> Option<MarkerName> {
    if let Some(i) = MARKER_NAMES.iter().position(|n| *n == name) {
        return Some(i as MarkerName);
    }
    EXTRA.lock().unwrap().iter().position(|n| *n == name).map(|i| (i + MARKER_NAMES.len()) as MarkerName)
}

/// Id for a marker name, registering it on first use. Ids are process-local;
/// the canonical state writes names.
pub fn intern(name: &'static str) -> MarkerName {
    let key = (name.as_ptr() as usize, name.len());
    if let Some(id) = IDS.with(|t| t.borrow().iter().find(|e| (e.0, e.1) == key).map(|e| e.2)) {
        return id;
    }
    let id = match MARKER_NAMES.iter().position(|n| *n == name) {
        Some(i) => i as MarkerName,
        None => {
            // Find or register under one lock, so every thread gets one id per name.
            let mut v = EXTRA.lock().unwrap();
            let i = match v.iter().position(|n| *n == name) {
                Some(i) => i,
                None => {
                    v.push(name);
                    v.len() - 1
                }
            };
            (MARKER_NAMES.len() + i) as MarkerName
        }
    };
    IDS.with(|t| t.borrow_mut().push((key.0, key.1, id)));
    id
}

/// `marker!("NAME")`: cached interned marker id.
#[macro_export]
macro_rules! marker {
    ($name:expr) => {{
        static ID: std::sync::OnceLock<$crate::markers::MarkerName> = std::sync::OnceLock::new();
        *ID.get_or_init(|| $crate::markers::intern($name))
    }};
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
