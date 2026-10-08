//! Palafin (TWM): Zero to Hero — once during your turn, when this Pokémon
//! moves from the Active Spot to the Bench, you may search your deck for a
//! Palafin ex and switch it with this Pokémon (the Palafin ex goes onto the
//! slot, this card goes into the deck, then shuffle). Wave Splash — 30.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Palafin@TWM",
    triggers: &[
        Trigger {
            origin: RuleSource::Ability,
            event: Event::OnMoved(OnMovedSpec { to: MovedTo::Bench }),
            steps: &[Step::new(Op::If(IfSpec {
                cond: Cond::Not(&Cond::HasMarker { who: Who::Me, name: "ABILITY_USED_MARKER", from: MarkerFrom::This }),
                yes: &[Step::new(Op::May(MaySpec {
                    asker: Who::Me,
                    when: Cond::True,
                    msg: "WANT_TO_USE_ABILITY",
                    // The use is recorded whatever the answer.
                    yes: &[
                        Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "ABILITY_USED_MARKER", source: RuleSource::Ability })),
                        Step::new(Op::If(IfSpec {
                            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                            yes: &[
                                Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Name("Palafin ex"), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_EVOLVE", ..PickSpec::DEFAULT })),
                                // Nothing chosen: nothing to put onto it.
                                Step::new(Op::SwapPokemonCard(SwapPokemonCardSpec { cards: 0, slot: SlotExpr::This, keep_index: false, into: ZoneRef(Who::Me, Zone::Deck), bottom: false })),
                                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                            ],
                            no: &[],
                        })),
                    ],
                    no: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "ABILITY_USED_MARKER", source: RuleSource::Ability }))],
                }))],
                no: &[],
            }))],
        },
        Trigger {
            origin: RuleSource::CardRule,
            event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
            steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "ABILITY_USED_MARKER", from: MarkerFrom::This }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
