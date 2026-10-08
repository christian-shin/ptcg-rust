//! Wally's Compassion (MEG / M1S): heal all damage from 1 of your Mega
//! Evolution Pokémon ex; if you do, put all Energy attached to it into your
//! hand.
//!
//! Twinleaf: only damaged Mega ex are selectable (the rest are blocked);
//! HealEffect for the slot's full damage, then every Energy card in the slot
//! (checked after the heal) moves to the hand in one MOVE_CARDS, whether or
//! not the heal did anything.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "WallysCompassion",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Damaged, SlotPred::Top(Pred::All(&[Pred::Tag(tag::POKEMON_SV_MEGA), Pred::Tag(tag::POKEMON_EX_LOWER)]))]))],
        // Heal all damage from 1 of your Mega Evolution Pokémon ex; if you do, put all Energy attached to it into your hand.
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Damaged, SlotPred::Top(Pred::All(&[Pred::Tag(tag::POKEMON_SV_MEGA), Pred::Tag(tag::POKEMON_EX_LOWER)]))])),
                msg: "CHOOSE_POKEMON_TO_HEAL",
            })),
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Chosen), hp: Num::DamageOn(SlotExpr::Chosen), via: HealVia::Effect, clear_conditions: false })),
            Step::new(Op::EnergyChoice(EnergyChoiceSpec { from: SlotTarget::Slot(SlotExpr::Chosen), how: EnergyHow::All { provided: false }, to: EnergyDest::Hand, ..EnergyChoiceSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
