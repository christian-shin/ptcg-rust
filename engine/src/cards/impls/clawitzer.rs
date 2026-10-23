//! Clawitzer (MEG 38): Fall Back to Reload - once during your turn, when this Pokémon moves from the Active Spot to your
//! Bench, you may use this Ability. Attach up to 2 Basic [W] Energy cards from your hand to this Pokémon. Aqua Launcher -
//! 210; discard all Energy from this Pokémon.
//!
//! Palafin TWM's trigger over the ChangeActive of this Pokémon to the Bench during your turn, with its usage record made
//! whatever the answer; asked only when a Basic [W] Energy card is in the hand; "up to 2" in an Ability is 1 to 2 cards
//! (id2301), then one Attach event per card onto this Pokémon (Bloodmoon Ursaluna SFA 25). Aqua Launcher discards every
//! Energy card after the damage (Cynthia's Garchomp ex ASC 111).
use crate::spec::prelude::*;

const WATER_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "Clawitzer",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::ChangeActive), EventPred::From(SlotPred::IsThisPokemon), EventPred::Turn(TurnOf::Me)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy))),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), WATER_ENERGY),
                yes: &[
                    Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy))),
                    Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: WATER_ENERGY, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, into: 0, ..PickSpec::DEFAULT })),
                    Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT })),
                ],
                no: &[Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy)))],
            }))],
            no: &[],
        }))],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
