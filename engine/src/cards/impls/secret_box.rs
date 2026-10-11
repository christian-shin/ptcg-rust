//! Secret Box (TWM, ACE SPEC Item): you can use this card only if you discard 3 other cards from your hand. Search your deck for
//! an Item card, a Pokémon Tool card, a Supporter card, and a Stadium card, reveal them, put them into your hand. Then,
//! shuffle your deck.
//!
//! The printed condition is 3 cards other than this one in the hand (`Cond::Printed`, so it binds the card when its effect is
//! used by another card too); the discard is a cost, `Step::cost` (APR E-30; id1124). The Search takes one of each kind or
//! fewer (none required: JP FAQ Secret Box) into the hand revealed; the shuffle follows (APR E-19). With an empty deck it
//! can't be used: the cost is paid for no effect (id1124, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SecretBox",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(3)))],
        steps: &[
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(3), max: Num::Lit(3) }, into: 0, ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Trainer, bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Tool), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Item), &Num::Lit(1))), &Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Stadium), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1)))) }, caps: &[Cap { kind: CapKind::Tool, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Tool), &Num::Lit(1)) }, Cap { kind: CapKind::Item, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Item), &Num::Lit(1)) }, Cap { kind: CapKind::Stadium, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Stadium), &Num::Lit(1)) }, Cap { kind: CapKind::Supporter, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Supporter), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
