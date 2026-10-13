//! N's PP Up (ASC 195, Item): attach a Basic Energy card from your discard pile to 1 of your Benched N's Pokémon.
//!
//! An Attach event (this Item its cause) onto a Benched Pokémon whose top card is an N's Pokémon (the `NS` tag).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsPPUp",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: SlotPred::Tag(crate::types::tag::NS), scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
             different_types: false, })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
