//! Chi-Yu (MEG 31): Scorching Earth - 40; if your opponent has a Stadium in play, discard it. If you do, your opponent can't
//! play any Stadium cards from their hand during their next turn.
//!
//! Like every Stadium discard of an attack it happens after the damage, before the Knock Out check: a LeavePlay of the Stadium to its owner's discard pile (D1), caused by this attack. "If you do" reads that
//! event's outcome (`Cond::Done`, APR E-20): a refused discard arms no lock. The lock is a lasting `Lock` on the opponent over
//! PlayTrainer of a Stadium from the hand (D14).
use crate::spec::prelude::*;

const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);

pub static SPEC: CardSpec = CardSpec {
    class: "ChiYuMEGPool",
    // Scorching Earth: if your opponent has a Stadium in play, discard it. If you do, your opponent
    // can't play any Stadium cards from their hand during their next turn.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
            yes: &[
                Step::new(Op::Discard(DiscardSpec { from: OPP_STADIUM, cards: CardSel::All, ..DiscardSpec::DEFAULT })),
                // "If you do": the Stadium's discard happened (a refused discard arms nothing).
                Step::new(Op::If(IfSpec {
                    cond: Cond::Done,
                    yes: &[Step::new(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(PLAY_STADIUM_FROM_HAND, "BLOCKED_BY_EFFECT"))) }))],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
