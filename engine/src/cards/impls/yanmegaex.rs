//! Yanmega ex (DRI 3): Buzzing Boost — once during your turn, when this Pokémon moves from your Bench to the Active Spot, you may search
//! your deck for up to 3 Basic [G] Energy cards and attach them to this Pokémon. Then, shuffle your deck. Jet Cyclone — 210; move 3
//! Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Buzzing Boost is a trigger over the ChangeActive event that brings this Pokémon to the Active Spot during its owner's turn (a
//! retreat, a switch, a promotion: APR E-25, whatever moved it), unless the Ability's usage record is there: a question; no → record; yes →
//! record, then up to 3 Basic [G] Energy from the deck onto it, then one shuffle (the printed "Then, shuffle your deck", APR E-19;
//! user decision D8). A blocked Ability asks nothing and sets nothing. Jet Cyclone moves min(3, Energy attached) Energy to one
//! Benched Pokémon (none to move with no Benched Pokémon: nothing more happens).
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
                // Not asked again once the record is made, and not offered while the Ability is blocked
                // (B-PC-17: a blocked Ability can't be used, so nothing is asked or marked).
                when: Cond::All(&[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy))), Cond::Not(&Cond::AbilityBlocked)]),
                yes: &[Step::new(Op::If(IfSpec {
                    cond: Cond::Not(&Cond::AbilityBlocked),
                    yes: &[
                        Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy))),
                        Step::new(Op::Attach(AttachSpec {
                            chooser: Who::Me,
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                            slots: AttachSlots::ActiveOnly,
                            target: SlotPred::Any, scan: TargetScan::InPlay,
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                            same_target: false,
                            different_targets: false,
                            valid_types: &[],
                            max_per_type: 0,
                            cancel: false,
                            onto: None,
                            cards: CardSel::All,
                         different_types: false, })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                }))],
                no: &[Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy)))],
            }))],
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
