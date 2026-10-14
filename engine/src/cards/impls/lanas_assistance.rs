//! Lana's Aid (TWM 155): put up to 3 in any combination of Pokémon that don't have a Rule Box and Basic Energy cards from your
//! discard pile into your hand.
//!
//! One Search op over the discard pile, a PutIntoHand from the discard pile (revealed by the move, id787; at least 1 when played from the
//! hand, id2199, id2301); used through Look-Alike Show it may take 0 (D5, APR D-05).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LanasAssistance",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)]), Pred::BasicEnergy]), bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
