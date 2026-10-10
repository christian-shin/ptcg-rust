//! Togekiss (SSP / ASC): Wonder Kiss - whenever your opponent's Active Pokémon gets Knocked Out, flip a coin; if heads,
//! take 1 more Prize card for that Knock Out (does not stack). Speed Wing - 140.
//!
//! An `On(KnockOut & Owner(Opp) & Slot(Active))` trigger of the Ability: any Knock Out of the opponent's Active counts,
//! whatever the cause (attack damage, Checkup Poison or Burn, a Pokémon Knocking itself Out: id2043; a Cursed Blast Knock
//! Out too, id2084, id2089: every effect resolves before the Knock Outs). One flip however many Togekiss
//! (id2083): while it is pending the Ability's marker on the player stops a second copy from flipping; heads adds 1 Prize to the
//! KnockOut (`Op::PrizeBonus`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Togekiss@SSP|ASC",
    // Wonder Kiss: whenever your opponent's Active Pokémon gets Knocked Out, flip a coin; if heads, take 1 more Prize card
    // for that Knock Out (does not stack: the marker is set on the player while the flip is pending).
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::KnockOut), EventPred::Owner(Who::Opp), EventPred::Slot(SlotPred::IsActive)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::HasMarker { who: Who::Opp, name: "TOGEKISS_KNOCKOUT_FLIP", from: MarkerFrom::Any },
            yes: &[],
            no: &[
                Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Opp), name: "TOGEKISS_KNOCKOUT_FLIP", source: RuleSource::CardRule })),
                Step::new(Op::Coin(CoinSpec {
                    heads: &[
                        Step::new(Op::PrizeBonus(PrizeBonusSpec { n: 1 })),
                        Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Opp), name: "TOGEKISS_KNOCKOUT_FLIP", from: MarkerFrom::Any })),
                    ],
                    tails: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Opp), name: "TOGEKISS_KNOCKOUT_FLIP", from: MarkerFrom::Any }))],
                    ..CoinSpec::DEFAULT
                })),
            ],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
