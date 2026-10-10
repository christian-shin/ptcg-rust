//! Meowth ex (POR 62): Last-Ditch Catch — once during your turn, when you play this Pokémon from your hand onto your
//! Bench, you may search your deck for a Supporter card, reveal it, and put it into your hand, then shuffle; no more
//! than 1 "Last-Ditch" Ability each turn. Tuck Tail — 60; put this Pokémon and all attached cards into your hand.
//!
//! Tuck Tail is one LeavePlay event (destination the hand) by the attack's effect, after the damage: Milotic TWM 50's
//! Mentally Calm on the opponent's side refuses it and Meowth ex stays with its cards (scenario
//! meowth-ex-por-62-tuck-tail-vs-milotic-twm-mentally-calm). Last-Ditch Catch is an `On(EnterPlay ..)` trigger with an
//! Ability origin; the "Last-Ditch" limit is a player marker cleared at the end of the turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Meowthex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Hand) }))] }],
    triggers: &[
        Trigger {
            origin: RuleSource::Ability,
            event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])),
            steps: &[Step::new(Op::If(IfSpec {
                // Only one "Last-Ditch" Ability per turn (the state marker); it needs a card in the deck.
                cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Not(&Cond::HasMarker { who: Who::Me, name: "TRUMP_CARD_MARKER", from: MarkerFrom::Any })]),
                yes: &[Step::new(Op::May(MaySpec {
                    asker: Who::Me,
                    when: Cond::True,
                    msg: "WANT_TO_USE_ABILITY",
                    yes: &[
                        Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "TRUMP_CARD_MARKER", source: RuleSource::CardRule })),
                        Step::new(Op::Search(SearchSpec {
                            pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                            destination: SearchDestination::Hand { reveal: true },
                            msg: "",
                            cancel: false,
                            shuffle_first: false,
                        })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                }))],
                no: &[],
            }))],
        },
        Trigger {
            origin: RuleSource::CardRule,
            event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
            steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "TRUMP_CARD_MARKER", from: MarkerFrom::Any }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
