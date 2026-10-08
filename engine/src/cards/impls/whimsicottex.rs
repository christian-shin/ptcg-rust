//! Whimsicott ex (SV11W 5): Energy Gift — search your deck for up to 3
//! Basic Energy cards and attach them to your Pokémon in any way you like,
//! then shuffle. Wonder Cotton — your opponent reveals their hand; 50
//! damage for each Trainer card there.
//!
//! Twinleaf quirks kept: an empty deck skips Energy Gift; the generator is
//! resumed inside the transfer loop, so with no transfer the deck is never
//! shuffled, and otherwise the ShuffleDeckPrompt (no wait) is created after
//! the first MOVE_CARDS (the rest follow before it resolves). Wonder Cotton
//! always shows the opponent's hand (even when empty) and sets
//! `effect.damage` when the prompt resolves.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Whimsicottex",
    attacks: &[
        AttackSpec {
            index: 0,
            // Energy Gift: attach up to 3 Basic Energy cards from your deck to your Pokémon in any way you like, then shuffle.
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Attach(AttachSpec {
                        chooser: Who::Me,
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::BasicEnergy,
                        slots: AttachSlots::BenchActive,
                        target: Pred::Any,
                        scan: TargetScan::InPlay,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        same_target: false,
                        different_targets: false,
                        valid_types: &[],
                        max_per_type: 0,
                        cancel: false,
                        route: AttachRoute::Move,
                        none_shuffles: false,
                     different_types: false, })),
                    // With nothing attached the deck is not shuffled (Twinleaf).
                    Step::new(Op::If(IfSpec { cond: Cond::Slot(SlotExpr::Picked, SlotPred::Any), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
                ],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            // Wonder Cotton: your opponent reveals their hand; 50 damage for each Trainer card there.
            steps: &[
                Step::before_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), to: Who::Me, when_empty: true })),
                Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Trainer), &Num::Lit(50)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
