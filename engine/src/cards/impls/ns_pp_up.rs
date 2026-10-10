//! N's PP Up (JTG): attach 1 Basic Energy from your discard pile to 1 of
//! your Benched N's Pokémon.
//!
//! Twinleaf: a Bench slot counts as "N's" if any card in its stack has the
//! tag; `blockedTo` lists every in-play Pokémon (Active included) whose top
//! card lacks the tag. The card is discarded by the prompt callback.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsPPUp",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::InPlayAny(Who::Me, PlayScope::Bench, Pred::Tag(crate::types::tag::NS))],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: Pred::Tag(crate::types::tag::NS),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                route: AttachRoute::Move,
                onto: None,
                cards: CardSel::All,
                none_shuffles: false,
             different_types: false, })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
