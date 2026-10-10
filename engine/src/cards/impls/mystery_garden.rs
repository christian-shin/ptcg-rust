//! Mystery Garden (MEG / ASC, stadium): once during each player's turn, that
//! player may discard 1 Energy card from their hand. If they do, that player
//! draws cards until they have as many cards in hand as they have [P]
//! Pokémon in play.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MysteryGarden",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[
            Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::Energy),
            // It must draw at least 1 card: the [P] Pokémon in play over the hand without the discarded Energy, up to the deck.
            Cond::Cmp(
                Num::Min(&Num::Sub(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(crate::types::ct::PSYCHIC)), &Num::Sub(&Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), &Num::Lit(1))), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck))),
                CmpOp::Gt,
                Num::Lit(0),
            ),
        ],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                    Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(crate::types::ct::PSYCHIC))) })),
                ],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
