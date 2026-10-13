//! Oricorio ex (PFL 18): Excited Turbo - as often as you like during your turn, if you have any [R] Mega Evolution Pokémon ex in
//! play, you may use this Ability. Attach a Basic [R] Energy card from your hand to 1 of your Benched [R] Pokémon. Fire Wing - 110.
//!
//! Each use attaches exactly 1 Basic [R] Energy card from the hand to a Benched Pokémon that is [R] now (`TargetScan::BenchEffectiveType`:
//! the current type, APR D-07); it needs a Mega Evolution Pokémon ex whose current type is [R] in play (`SlotPred::TypeIs`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Oricorioex",
    attacks: &[],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        // A [R] Mega Evolution Pokémon ex in play: the type is the Pokémon's current one.
        needs: &[Cond::AnySlot(
            SlotSel::Pokemon(Who::Me),
            SlotPred::All(&[SlotPred::Top(Pred::All(&[Pred::PokemonEx, Pred::Tag(crate::types::tag::POKEMON_SV_MEGA)])), SlotPred::TypeIs(crate::types::ct::FIRE)]),
        )],
        steps: &[Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(crate::types::ct::FIRE),
                scan: TargetScan::BenchEffectiveType,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
             different_types: false, }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
