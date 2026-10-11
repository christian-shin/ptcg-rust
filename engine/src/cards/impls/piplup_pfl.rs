//! Piplup (PFL 27): Call for Support: search your deck for a Supporter card, reveal it, and put it into your hand. Then,
//! shuffle your deck. Tackle: 20.
//!
//! The chosen Supporter is revealed before it moves (id1131) and goes into the hand (PutIntoHand, this attack its cause); an
//! empty deck can't be searched (id840), otherwise the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Piplup@PFL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
