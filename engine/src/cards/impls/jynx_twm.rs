//! Jynx (TWM 46): Inviting Kiss — search your deck for a Basic Pokémon and put it onto your Bench. Then, shuffle your deck. If
//! you put any Pokémon onto your Bench in this way, move an Energy from this Pokémon to the new Benched Pokémon. Icy Snow — 30.
//!
//! A deck search, asked after the damage (the deck is hidden until it resolves); with no room on the Bench nothing is searched
//! (Call for Family). "The new Benched Pokémon" is the Pokémon the card found is (`SlotExpr::Chosen`: the search's register);
//! the Energy is chosen when the move is reached, only when that Pokémon is in play ("If you put any Pokémon onto your Bench in
//! this way", APR E-20), and moves between your own Pokémon (a MoveEnergy event, this attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "JynxTWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::BenchSpace(Who::Me),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Slot(SlotExpr::Chosen(0), SlotPred::Any),
                    yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                        target: SlotTarget::Slot(SlotExpr::This),
                        selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: true },
                        to: EnergyDest::Slot(SlotExpr::Chosen(0)),
                        ..DiscardEnergySpec::DEFAULT
                    }))],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
