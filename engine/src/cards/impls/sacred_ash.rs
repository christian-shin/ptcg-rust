//! Sacred Ash (FLF 96): printed "shuffle 5 Pokémon from your discard pile into your deck"; the erratum is "up to 5"
//! (id2176; JP FAQ Sacred Ash せいなるはい: with 5 Pokémon in the discard pile you may choose just 1: 「はい、できます。」).
//! The same program as the POR 115 print (`sacred_ash_dri.rs`): an Item's "up to" is 1 to 5 (id2301); the chosen Pokémon
//! are shown and put into the deck by one PutIntoDeck event, then the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SacredAsh@FLF",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(5)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Deck { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
