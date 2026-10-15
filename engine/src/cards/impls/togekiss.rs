//! Togekiss (SSP / ASC): Wonder Kiss - whenever your opponent's Active Pokémon gets Knocked Out, flip a coin; if heads,
//! take 1 more Prize card for that Knock Out (does not stack). Speed Wing - 140.
//!
//! An `On(KnockOut & Owner(Opp) & Slot(Active))` trigger of the Ability: any Knock Out of the opponent's Active counts,
//! whatever the cause (attack damage, Checkup Poison or Burn, a Pokémon Knocking itself Out: id2043; a Cursed Blast Knock
//! Out too, id2084, id2089: every effect resolves before the Knock Outs). One flip however many Togekiss
//! (id2083): the Ability's usage record of the Knocked Out Pokémon stops a second copy from flipping; heads adds 1 Prize to the
//! KnockOut (`Op::PrizeBonus`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Togekiss",
    // Wonder Kiss: whenever your opponent's Active Pokémon gets Knocked Out, flip a coin; if heads, take 1 more Prize card
    // for that Knock Out (does not stack: the usage record of the Knocked Out Pokémon, id2083).
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::KnockOut), EventPred::Owner(Who::Opp), EventPred::Slot(SlotPred::IsActive)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Used(UsedSpec { ability: 0, by: UsedBy::Picked, until: UsedUntil::Removed }),
            yes: &[],
            no: &[
                Step::new(Op::Used(UsedSpec { ability: 0, by: UsedBy::Picked, until: UsedUntil::Removed })),
                Step::new(Op::Coin(CoinSpec { heads: &[Step::new(Op::PrizeBonus(PrizeBonusSpec { n: 1 }))], ..CoinSpec::DEFAULT })),
            ],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
