//! Explorer's Guidance (TEF 147, Ancient Supporter): look at the top 6 cards of your deck and put 2 of them into your hand.
//! Discard the other cards.
//!
//! A Look of the top 6 (set aside in the scratch zone 1; not a search: no shuffle); exactly min(2, cards looked at) are
//! chosen and put into the hand (PutIntoHand); the others are discarded (a Discard event). Played from the hand it is the turn's
//! Supporter play, recorded by its PlayTrainer event (the played-this-turn record that "if you played an Ancient Supporter
//! card from your hand this turn" reads, APR E-26); its effect used by an attack isn't a play (id2225).
use crate::spec::prelude::*;

const LOOKED_AT: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));

pub static SPEC: CardSpec = CardSpec {
    class: "ExplorersGuidance",
    // Look at the top 6 cards of your deck and put 2 of them into your hand. Discard the other cards.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: LOOKED_AT,
                bounds: Bounds { min: Num::Min(&Num::Lit(2), &Num::ZoneSize(LOOKED_AT)), max: Num::Lit(2) },
                into: 0,
                purpose: Some(PromptPurpose::PutIntoHand),
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: LOOKED_AT, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: LOOKED_AT, cards: CardSel::All, ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
