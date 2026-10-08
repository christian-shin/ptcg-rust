//! Secret Box (TWM, ACE SPEC): discard 3 other cards from your hand; search
//! your deck for an Item, a Pokémon Tool, a Supporter and a Stadium, reveal
//! them, put them into your hand, then shuffle.
//!
//! Twinleaf quirks kept: a MOVE_CARDS hand->supporter of the card (already in
//! the supporter pile) is reduced; the final ShuffleDeckPrompt has no trailing
//! WaitPrompt. Fixed (phase 4b, R3): choosing nothing from the deck no longer
//! skips the shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SecretBox",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Ge, Num::Lit(3))],
        steps: &[
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(3), max: Num::Lit(3) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Trainer, bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Tool), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Item), &Num::Lit(1))), &Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Stadium), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1)))) }, caps: &[Cap { kind: CapKind::Tool, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Tool), &Num::Lit(1)) }, Cap { kind: CapKind::Item, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Item), &Num::Lit(1)) }, Cap { kind: CapKind::Stadium, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Stadium), &Num::Lit(1)) }, Cap { kind: CapKind::Supporter, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
