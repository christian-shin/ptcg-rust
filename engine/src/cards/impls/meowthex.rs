//! Meowth ex (M3 / POR): Last-Ditch Catch — when played from hand onto the
//! Bench, you may search your deck for a Supporter (Twinleaf: any Trainer,
//! non-Supporters blocked). Tuck Tail — put this Pokémon and all attached
//! cards into your hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Meowthex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Hand), effect_of_attack: false }))] }],
    triggers: &[
        Trigger {
            origin: RuleSource::Ability,
            event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }),
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
