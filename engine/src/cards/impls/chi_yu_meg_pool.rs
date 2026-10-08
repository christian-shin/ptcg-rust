//! Chi-Yu (MEG 31): Scorching Earth — 40; if your opponent has a Stadium in
//! play, discard it. If you do, your opponent can't play any Stadium cards
//! from their hand during their next turn.
//!
//! Twinleaf: the Stadium is discarded only when it sits on the opponent's
//! side (MOVE_CARDS stadium → owner's discard); if it left play, a
//! PlayLockEffect `{ stadium: true }` follows.
//!
//! Fixed (phase 4b, R7F-4; ruling 1589): the Stadium was discarded in the
//! attack handler, before the damage; like every Stadium discard of an
//! attack it now happens in AfterAttackEffect (after the damage, before the
//! Knock Out check), and the play lock is built from a fresh AttackEffect's
//! data like the other AfterAttackEffect handlers do.
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
                Step::new(Op::Move(MoveSpec { from: OPP_STADIUM, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Not(&Cond::Nonempty(OPP_STADIUM, Pred::Any)),
                    yes: &[Step::new(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(Locked::Stadium) }))],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
