//! Deduction Kit (SSP 171, Item): look at the top 3 cards of your deck and put them back in any order, or shuffle them and put
//! them on the bottom of your deck.
//!
//! The Look sets the top 3 cards aside (all of the deck when it holds fewer), then the player chooses one of two puts of
//! them back into the deck: on top in the order the player chooses (`DeckOrder::ChosenBy`), or shuffled among themselves on the
//! bottom (`DeckOrder::Shuffled`, APR E-35: "shuffle them and put them on the bottom" shuffles only those cards, not the
//! deck). With an empty deck there is nothing to look at, so nothing is set aside to put back and the card can't be played
//! (APR B-01 step 1, id925): the rules' walk follows the Look's cards into each option of the choice.
use crate::spec::prelude::*;

const SET_ASIDE: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(0));

pub static SPEC: CardSpec = CardSpec {
    class: "DeductionKit",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(3)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Choose(ChooseSpec {
                chooser: Who::Me,
                options: &[
                    ChoiceBranch {
                        label: "PUT_BACK_IN_ANY_ORDER",
                        avail: Cond::True,
                        body: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: SET_ASIDE, cards: CardSel::All, position: DeckPosition::Top, order: DeckOrder::ChosenBy(Who::Me), ..PutIntoDeckSpec::DEFAULT }))],
                    },
                    ChoiceBranch {
                        label: "SHUFFLE_AND_PUT_ON_THE_BOTTOM",
                        avail: Cond::True,
                        body: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: SET_ASIDE, cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT }))],
                    },
                ],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
