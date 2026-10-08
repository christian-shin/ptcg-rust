//! Rosa's Encouragement (POR / M3): only if you have more Prize cards
//! remaining than your opponent; attach up to 2 Basic Energy cards from your
//! discard pile to 1 of your Stage 2 Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RosasEncouragement",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[
            Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::PrizesLeft(Who::Opp)),
            Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::StageIs(crate::types::Stage::Stage2)),
        ],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Discard),
            predicate: Pred::BasicEnergy,
            slots: AttachSlots::ActiveBench,
            target: Pred::StageIs(crate::types::Stage::Stage2),
            // "Up to 2" takes at least 1 when played from the hand; through an attack it may be 0.
            bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Min(&Num::Lit(2), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy)) },
            same_target: true,
            ..AttachSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
