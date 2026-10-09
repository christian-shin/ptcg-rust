//! Bianca's Devotion (TEF): heal all damage from 1 of your Pokémon that has
//! 30 HP or less remaining.
use crate::spec::prelude::*;

const HEALABLE: SlotPred = SlotPred::All(&[SlotPred::Damaged, SlotPred::RemainingHpAtMost(30)]);

pub static SPEC: CardSpec = CardSpec {
    class: "BiancasDevotion",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), HEALABLE)],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), HEALABLE),
                msg: "CHOOSE_POKEMON_TO_HEAL",
            })),
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::DamageOn(SlotExpr::Picked), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
