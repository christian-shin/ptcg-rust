//! Huntail (DRI 55): Diver's Catch - when 1 of your [W] Pokémon is Knocked Out by damage from an attack from your opponent's Pokémon,
//! you may put all Basic [W] Energy attached to that Pokémon into your hand instead of the discard pile. Wave Splash - 80.
//!
//! A trigger on the KnockOut of any of your [W] Pokémon (this one included, Active or Benched) by damage from an attack of the
//! opponent's Pokémon, run while it is still in play (APR D step 2, like Heavy Baton): asked when the Ability isn't blocked and a
//! Basic [W] Energy card is attached; on yes every Basic [W] Energy card of that Pokémon leaves play for the hand (a LeavePlay caused
//! by the Ability), the other attached cards go to the discard pile with the Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Huntail",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::KnockOut),
            EventPred::Owner(Who::Me),
            EventPred::Slot(SlotPred::TypeIs(ct::WATER)),
            EventPred::KoBy(KoBy::AttackDamage),
            EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
        ])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::Slot(SlotExpr::Picked, SlotPred::HasCard(Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::WATER)])))]),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                    target: SlotTarget::Slot(SlotExpr::Picked),
                    selection: EnergySelection::All { provided: false, kind: EnergyKind::BasicProvides(ct::WATER) },
                    to: EnergyDest::Hand,
                    ..DiscardEnergySpec::DEFAULT
                }))],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
