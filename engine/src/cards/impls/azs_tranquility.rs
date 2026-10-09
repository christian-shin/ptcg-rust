//! AZ's Tranquility (CRI / M4): switch your Active Pokémon with 1 of your
//! Benched Pokémon; if you moved a Pokémon ex to the Bench, heal 80 from it.
//!
//! Twinleaf: the Supporter moves to the supporter pile with preventDefault,
//! then a mandatory ChoosePokemon prompt on the Bench; the callback switches
//! and heals the previous Active (any card tagged ex in the slot) by 80.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AzsTranquility",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: true })),
            // The Pokémon that moved to the Bench (the switch leaves it in the slot register): if it
            // is a Pokémon ex, heal 80 from it.
            Step::new(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Picked, SlotPred::AnyCardTag(crate::types::tag::POKEMON_EX_LOWER)),
                yes: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(80), clear_conditions: false }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
