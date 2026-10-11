//! Dudunsparce (TEF / PRE): Run Away Draw — draw 3 cards, then shuffle this Pokémon and all attached cards into your deck.
//!
//! The Draw is a Draw event of the Ability's effect; then the Pokémon and every card attached leave play into the deck
//! (a LeavePlay, caused by the Ability) and the deck is shuffled. Needs a card in the deck (id2231).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dudunsparce@TEF|PRE",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Deck) })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
