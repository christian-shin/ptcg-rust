//! Wally's Compassion (MEG 132 / M1S, Supporter): heal all damage from 1 of your Mega Evolution Pokémon ex. If you healed any
//! damage in this way, put all Energy attached to that Pokémon into your hand.
//!
//! The Heal and the move are separate events: "if you healed" reads the Heal's outcome (`Cond::Done`, APR E-20), so a refused heal
//! moves no Energy. The Energy go to the hand as a LeavePlay of each attached card (to the hand, D1). Usability needs a damaged
//! Mega ex (an effect must change the game).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "WallysCompassion",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        // Heal all damage from 1 of your Mega Evolution Pokémon ex; if you do, put all Energy attached to it into your hand.
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Damaged, SlotPred::Top(Pred::All(&[Pred::Tag(tag::POKEMON_SV_MEGA), Pred::PokemonEx]))])) }, purpose: PromptPurpose::HealTarget })),
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::DamageOn(SlotExpr::Picked), clear_conditions: false })),
            // "If you healed any damage in this way": the Heal's outcome.
            Step::new(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::All { provided: false, kind: EnergyKind::Any }, to: EnergyDest::Hand, ..DiscardEnergySpec::DEFAULT }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
