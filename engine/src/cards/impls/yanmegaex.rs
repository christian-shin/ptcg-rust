//! Yanmega ex (DRI): Buzz Boost — once during your turn, when this Pokémon
//! moves from your Bench to the Active Spot, you may search your deck for up
//! to 3 Basic [G] Energy and attach them to this Pokémon, then shuffle.
//! Jet Cyclone — 210; move 3 Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Buzz Boost is a trigger over the ChangeActive event that brings this Pokémon
//! to the Active Spot during its owner's turn (a retreat, a switch, a
//! promotion: APR E-25, whatever moved it), unless the player marker is set:
//! a question; no → marker; yes → marker, then up to 3 Basic [G] Energy from
//! the deck onto it (one shuffle per card attached, as recorded). Jet Cyclone: AttachEnergyPrompt (Active →
//! Bench, any Energy, sameTarget, no cancel), MOVE_CARDS each. Phase 4b fix:
//! with no Benched Pokémon the attack does nothing more (the prompt, min 3,
//! was unanswerable), and min = max = min(3, Energy attached) so a Pokémon
//! with fewer than 3 Energy cards can't get stuck either.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yanmegaex",
    triggers: &[
        // Buzz Boost: once during your turn, when this Pokémon moves from your Bench to the Active Spot, you may search your
        // deck for up to 3 Basic [G] Energy and attach them to this Pokémon, then shuffle.
        Trigger {
            origin: RuleSource::Ability,
            event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::ChangeActive), EventPred::To(SlotPred::IsThisPokemon), EventPred::Turn(TurnOf::Me)])),
            steps: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                // Not asked again once the marker is set, and not offered while the Ability is blocked
                // (B-PC-17: a blocked Ability can't be used, so nothing is asked or marked).
                when: Cond::All(&[Cond::Not(&Cond::HasMarker { who: Who::Me, name: "BUZZ_BOOST_MARKER", from: MarkerFrom::This }), Cond::Not(&Cond::AbilityBlocked)]),
                msg: "WANT_TO_USE_ABILITY",
                yes: &[Step::new(Op::If(IfSpec {
                    cond: Cond::Not(&Cond::AbilityBlocked),
                    yes: &[
                        Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "BUZZ_BOOST_MARKER", source: RuleSource::Ability })),
                        Step::new(Op::Attach(AttachSpec {
                            chooser: Who::Me,
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                            slots: AttachSlots::ActiveOnly,
                            target: Pred::Any,
                            scan: TargetScan::InPlay,
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                            same_target: false,
                            different_targets: false,
                            valid_types: &[],
                            max_per_type: 0,
                            cancel: false,
                            // Twinleaf shuffles after each card moved (and once when none was).
                            route: AttachRoute::MoveShufflePerCard,
                            none_shuffles: true,
                         different_types: false, })),
                    ],
                    no: &[],
                }))],
                no: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "BUZZ_BOOST_MARKER", source: RuleSource::Ability }))],
            }))],
        },
        Trigger {
            origin: RuleSource::CardRule,
            event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
            steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "BUZZ_BOOST_MARKER", from: MarkerFrom::This }))],
        },
    ],
    attacks: &[AttackSpec {
        index: 0,
        // Jet Cyclone: move 3 Energy from this Pokémon to 1 of your Benched Pokémon (all of them when it has fewer).
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::ToBench {
                min: Num::Min(&Num::Lit(3), &Num::CardCount(ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE)), Pred::Energy)),
                max: Num::Min(&Num::Lit(3), &Num::CardCount(ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE)), Pred::Energy)),
                same_target: true,
                via_effect: false,
                kind: EnergyKind::Any,
            }, to: EnergyDest::Stay, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
