//! Paldean Tauros (SSP 39): Upthrusting Horns — 30; you may put 2 Energy
//! attached to your opponent's Active Stage 2 Pokémon into their hand.
//! Jet Headbutt — 100.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PaldeanTaurosSSP39",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            // Only against a Stage 2 Pokémon that has Energy to take.
            when: Cond::All(&[
                Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::StageIs(crate::types::Stage::Stage2))),
                Cond::Cmp(Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), CmpOp::Gt, Num::Lit(0)),
            ]),
            yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: crate::types::ct::COLORLESS, up_to: true }, to: EnergyDest::Hand, ..DiscardEnergySpec::DEFAULT }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
