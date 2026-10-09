//! AZ's Tranquility (CRI / M4): switch your Active Pokémon with 1 of your
//! Benched Pokémon. If you moved a Pokémon ex to your Bench in this way,
//! heal 80 damage from it.
//!
//! The switch is a ChangeActive (Switch, APR C-03) done by this Supporter to
//! your Active Pokémon. "If you moved ... in this way" is an outcome of that
//! change, like "if you do": the heal needs the change done (`Cond::Done`)
//! and the Pokémon it moved to the Bench to be a Pokémon ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AzsTranquility",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: true })),
            // The switch was done and the Pokémon it moved to the Bench (the switch leaves it in the slot
            // register) is a Pokémon ex: heal 80 from it.
            Step::new(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Done, Cond::Slot(SlotExpr::Picked, SlotPred::AnyCardTag(crate::types::tag::POKEMON_EX_LOWER))]),
                yes: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(80), clear_conditions: false }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
