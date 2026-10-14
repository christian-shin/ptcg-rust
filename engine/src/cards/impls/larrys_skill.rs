//! Larry's Skill (PRE 115): discard your hand and search your deck for a Pokémon, a Supporter card, and a Basic Energy card,
//! reveal them, and put them into your hand. Then, shuffle your deck.
//!
//! Discarding the hand is a cost (APR E-30; id1124: the Supporter can't be played for no net effect), done before the search (a
//! Discard event over the hand); one card per kind (at most 1 Pokémon, 1 Supporter, 1 Basic Energy card) is revealed and put
//! into the hand (PutIntoHand), then the deck is shuffled (APR E-19).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LarrysSkill",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..DiscardSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::OneOf(&[Pred::Pokemon, Pred::Supporter, Pred::BasicEnergy]), bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Pokemon), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1))), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::BasicEnergy), &Num::Lit(1))) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Pokemon), &Num::Lit(1)) }, Cap { kind: CapKind::Supporter, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1)) }, Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::BasicEnergy), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
