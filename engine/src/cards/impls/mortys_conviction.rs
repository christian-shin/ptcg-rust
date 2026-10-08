//! Morty's Conviction (TEF, supporter): discard another card from your hand,
//! then draw a card for each of your opponent's Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MortysConviction",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::BenchCount(Who::Opp)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
