//! Dudunsparce (TEF, PRE): Run Away Draw — draw 3 cards, then shuffle this
//! Pokémon and all attached cards into your deck. Needs a card in the deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dudunsparce@Dudunsparce TEF|Dudunsparce PRE",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Deck) })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
