//! Night Stretcher (SFA): put a Pokémon or a Basic Energy card from your
//! discard pile into your hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NightlyStretcher",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::OneOf(&[Pred::Pokemon, Pred::BasicEnergy]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                // The prompt lists how many of each kind can be taken.
                caps: &[
                    Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(1)) },
                    Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy), &Num::Lit(1)) },
                ],
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
