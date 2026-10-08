//! Togekiss (SSP / ASC): Wonder Kiss - whenever your opponent's Active Pokémon
//! gets Knocked Out, flip a coin; if heads, take 1 more Prize card for that
//! Knock Out (does not stack). Speed Wing - 140.
//!
//! Twinleaf: on a KnockOutEffect for the owner's Active, with this card in
//! play on the other side, unless the Ability is blocked and the sourceless
//! marker TOGEKISS_KNOCKOUT_FLIP isn't set: set the marker, flip (the
//! KnockOutEffect is retained across the flip), `prizeCount += 1` on heads
//! when it is > 0, then remove the marker.
//!
//! Fixed (phase 4b, R7F-1; rulings 1591, 1619, 1623): the handler used to
//! require the ATTACK phase of Togekiss' owner, so a Knock Out by Poison or
//! Burn in Pokémon Checkup, or by an Ability (Cursed Blast), took no extra
//! Prize. Any Knock Out of the opponent's Active counts.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Togekiss@SSP|ASC",
    // Wonder Kiss: whenever your opponent's Active Pokémon gets Knocked Out, flip a coin; if heads, take 1 more Prize card
    // for that Knock Out (does not stack: the marker is set on the player while the flip is pending).
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnKnockOut(OnKnockOutSpec { which: KoWhich::OppActive }),
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
