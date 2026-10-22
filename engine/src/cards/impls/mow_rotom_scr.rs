//! Mow Rotom (SCR 8): Reaping Dash - 30; before doing damage, discard all Pokémon Tools and Special Energy from your opponent's
//! Active Pokémon.
//!
//! Both leave play before the damage, as the text says (the "Energy leaves after the damage" default is for an effect that
//! doesn't say "before"): each of the Defending Pokémon's Special Energy cards, then its Tools, is a LeavePlay caused by this
//! attack (Megaton Blower's form), which Mist Energy and the like stop for their Pokémon.
use crate::spec::prelude::*;

const OPP_ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::Attached(OPP_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "MowRotomSCR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Snapshot(SnapshotSpec { zone: OPP_ATTACHED, predicate: Pred::SpecialEnergy, into: 0 })),
            Step::before_damage(Op::Discard(DiscardSpec { from: OPP_ATTACHED, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(OPP_ACTIVE)), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
