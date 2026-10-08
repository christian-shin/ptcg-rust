//! Litwick (PBL / M5): Will-O-Wisp - 20. No effects (Twinleaf's
//! `reduceEffect` only returns the state).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec { class: "Litwick", ..CardSpec::NONE };

pub static IMPL: CardImpl = SPEC.card_impl();
