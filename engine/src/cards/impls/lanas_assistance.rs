//! Lana's Aid (TWM, supporter): put up to 3 in any combination of Pokémon
//! without a Rule Box and Basic Energy cards from your discard pile into
//! your hand.
//!
//! Twinleaf: DiscardToHandEffect is probed before the supporter check; the
//! card moves to the supporter pile before the "no target" failure; the
//! cards are shown to the opponent and then moved.
//!
//! R7C: as the effect of an attack (Mr. Mime's Look-Alike Show) the prompt is min 0
//! (rulings 1844, 1853); played from the hand it is min 1.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LanasAssistance",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)]), Pred::BasicEnergy]), bounds: Bounds { min: Num::If(&Cond::ViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
