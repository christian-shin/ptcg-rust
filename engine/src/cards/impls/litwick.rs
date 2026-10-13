//! Litwick (PBL / M5): Will-O-Wisp - 20. No Ability and no effect text: the attack's damage is all there is.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec { class: "Litwick", ..CardSpec::NONE };

pub static IMPL: CardImpl = SPEC.card_impl();
