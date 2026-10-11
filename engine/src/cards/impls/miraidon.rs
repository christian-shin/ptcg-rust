//! Miraidon (TEF 121): Peak Acceleration - 40; search your deck for up to 2 Basic Energy cards and attach them to your
//! Future Pokémon in any way you like. Then, shuffle your deck. Sparking Strike - 160.
//!
//! The Energy is attached from the deck to the Future Pokémon in play (Active or Benched): one Attach event per card
//! (cause: this attack); an attack's "up to 2" may be 0 (id2301). The deck is shuffled once after the attaching (APR E-19,
//! D9), also when nothing was attached.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miraidon",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: Pred::Tag(crate::types::tag::FUTURE),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
             different_types: false, })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
