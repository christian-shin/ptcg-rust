//! Heavy Baton (TEF / PAR, tool): if the Pokémon this card is attached to has
//! a Retreat Cost of exactly 4, is in the Active Spot, and is Knocked Out
//! by damage from an attack from your opponent's Pokémon, move up to 3 Basic
//! Energy cards from that Pokémon to your Benched Pokémon in any way you
//! like.
//!
//! The Active Spot and the Retreat Cost are read when the damage is done (id1992; JP FAQ Heavy Baton): each Damage event
//! of an opponent's attack on the holder records whether it is Active with a Retreat Cost of exactly 4 (a marker on the
//! Pokémon, kept when it moves to the Bench; the latest damage decides, so damage done on the Bench clears it). At its KnockOut by damage from an attack from the opponent's Pokémon
//! (APR D step 2, while it is still in play) with the record, its owner moves 1 to 3 of its Basic Energy cards to their
//! Benched Pokémon (MoveEnergy events by the Tool; nothing without a Benched Pokémon; the JP FAQ: no choice to decline,
//! not by devolving).
use crate::spec::prelude::*;

const ACTIVE_AT_DAMAGE: &str = "HEAVY_BATON_ACTIVE_MARKER";
const BY_OPP_ATTACK: CausePred = CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)]);

pub static SPEC: CardSpec = CardSpec {
    class: "HeavyBaton",
    triggers: &[
        Trigger {
            origin: RuleSource::Tool,
            event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Damage), EventPred::Slot(SlotPred::Holder), EventPred::Cause(BY_OPP_ATTACK)])),
            steps: &[Step::new(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Slot(SlotExpr::Picked, SlotPred::IsActive), Cond::Cmp(Num::RetreatCostColorless(Who::Me), CmpOp::Eq, Num::Lit(4))]),
                yes: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Slot(SlotExpr::Picked), name: ACTIVE_AT_DAMAGE, source: RuleSource::TrainerEffect }))],
                no: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Slot(SlotExpr::Picked), name: ACTIVE_AT_DAMAGE, from: MarkerFrom::This }))],
            }))],
        },
        Trigger {
            origin: RuleSource::Tool,
            event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::KnockOut), EventPred::Slot(SlotPred::Holder), EventPred::KoBy(KoBy::AttackDamage), EventPred::Cause(BY_OPP_ATTACK)])),
            steps: &[Step::new(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Picked, SlotPred::MarkerFromThis(ACTIVE_AT_DAMAGE)),
                yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                    target: SlotTarget::Slot(SlotExpr::Picked),
                    selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(3), same_target: false, via_effect: false, kind: EnergyKind::Basic },
                    to: EnergyDest::Stay,
                    ..DiscardEnergySpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
