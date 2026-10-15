//! Mandibuzz (WHT 64): Look for Prey: once during your turn, you may use this Ability. Your opponent reveals their hand, and you
//! put a Basic Pokémon with 70 HP or less that you find there onto your opponent's Bench. Cutting Wind: 90.
//!
//! An Ability (once per turn). Usable when the opponent has a card in hand and an empty Bench spot (their open Bench is public
//! knowledge). The chosen Basic Pokémon (70 HP or less) is put onto the opponent's Bench from their hand (an EnterPlay event,
//! this Ability its cause); the hand is revealed first (a Reveal), with or without such a Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mandibuzz",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        // Their open Bench is public knowledge: with it full the Ability can't be used.
        needs: &[Cond::BenchSpace(Who::Opp)],
        // "Your opponent reveals their hand, and you put a Basic Pokémon ... that you find there onto your opponent's Bench."
        steps: &[Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })), Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::All(&[Pred::Basic, Pred::HpAtMost(70)])),
            yes: &[
                Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::All(&[Pred::Basic, Pred::HpAtMost(70)]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
                Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 0, who: Who::Opp })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
