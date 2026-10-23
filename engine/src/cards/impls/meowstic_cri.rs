//! Meowstic (CRI 37): Tricky Steps — 80; you may move an Energy from your opponent's Active Pokémon to 1 of their Benched
//! Pokémon.
//!
//! The same text as Team Rocket's Zapdos DRI 70's Jamming Wing, the same form (one rule per phrase): asked first (a "may", only
//! with an opponent's Benched Pokémon and an Energy on their Active Pokémon), then the Energy moves between the opponent's Pokémon
//! as an effect of this attack (a MoveEnergy event: Mist Energy on the Defending Pokémon keeps its Energy there, id2375; moved onto
//! a Pokémon with Mist Energy it is discarded, JP FAQ Elgyem BLK 40 / Mist Energy).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MeowsticCRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any), Cond::Slot(OPP_ACTIVE, SlotPred::HasEnergy)]),
            yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(1), same_target: false, via_effect: true, kind: EnergyKind::Any },
                to: EnergyDest::Stay,
                ..DiscardEnergySpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
