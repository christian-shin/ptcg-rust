//! Gengar (POR 50): Infinite Shadow - if this Pokémon is Knocked Out by damage from an attack from your opponent's Pokémon, put it into
//! your hand instead of the discard pile. (Discard all attached cards.) Mind Jack - 10+; this attack does 30 more damage for each of
//! your opponent's Benched Pokémon.
//!
//! Infinite Shadow is a trigger on this card's own KnockOut by damage from an attack of the opponent's Pokémon, run while it is still
//! in play (APR D step 2, as Pecharunt ASC 143's Final Chain): the attached Energy cards and Tools are discarded, then the Pokémon
//! (its whole stack of Pokémon cards: id52) leaves play for the hand (a LeavePlay caused by the Ability). The Knock Out stays a Knock
//! Out: the Prize cards are taken as usual. When the move to the hand is refused (Milotic TWM 50's Mentally Calm) the Pokémon goes to
//! the discard pile with the state check. Mind Jack: 30 more for each Benched Pokémon of the opponent's, before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gengar",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::BenchCount(Who::Opp), &Num::Lit(30)), when: Cond::True }))],
    }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::KnockOut),
            EventPred::This(Role::Card),
            EventPred::KoBy(KoBy::AttackDamage),
            EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
        ])),
        steps: &[
            Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::All { provided: false }, ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Tools(SlotExpr::This)), cards: CardSel::All, into: None })),
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Hand), top: false })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
