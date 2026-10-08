//! Cook (FST): heal 70 damage from your Active Pokémon.
//!
//! Twinleaf reduces a HealEffect on `player.active` and lets the Trainer
//! play continue normally.
//! Fixed (phase 4b, R3): can't be played while the Active has no damage (a
//! Trainer with no possible effect is unplayable; Rulings Compendium 851).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cook",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Slot(MY_ACTIVE, SlotPred::Damaged)],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(70), via: HealVia::Effect, clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
