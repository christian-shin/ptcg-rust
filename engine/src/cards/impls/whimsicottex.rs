//! Whimsicott ex (SV11W 5): Energy Gift — search your deck for up to 3
//! Basic Energy cards and attach them to your Pokémon in any way you like,
//! then shuffle. Wonder Cotton — your opponent reveals their hand; 50
//! damage for each Trainer card there.
//!
//! An empty deck skips Energy Gift; after a search of a nonempty deck the deck
//! is shuffled even when no Energy is chosen (ruling 2303). Wonder Cotton
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
                    // Then shuffle, even when no Energy was chosen (ruling 2303).
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
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
