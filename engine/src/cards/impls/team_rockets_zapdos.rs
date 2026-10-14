//! Team Rocket's Zapdos (DRI 70): Jamming Wing: 30; you may move an Energy from your opponent's Active Pokémon to 1 of their
//! Benched Pokémon. Wicked Thunder: 60+; if this Pokémon has Team Rocket's Energy attached, this attack does 60 more damage.
//!
//! Jamming Wing asks first (a "may", only with an opponent's Benched Pokémon and an Energy on their Active); the Energy moves
//! between the opponent's Pokémon (a MoveEnergy event onto their Benched Pokémon, this attack its cause, so Mist Energy on
//! the Defending Pokémon refuses it, id2375).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsZapdos",
    attacks: &[
        AttackSpec {
            index: 0,
            // You may move an Energy from your opponent's Active Pokémon to 1 of their Benched Pokémon.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any), Cond::Slot(OPP_ACTIVE, SlotPred::HasEnergy)]),
                purpose: PromptPurpose::Activate,
                yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(1), same_target: false, via_effect: true, kind: EnergyKind::Any }, to: EnergyDest::Stay, ..DiscardEnergySpec::DEFAULT }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(60, Cond::Slot(MY_ACTIVE, SlotPred::HasEnergyNamed("Team Rocket's Energy"))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
