//! Wondrous Patch (Wonder Patch MBD / PFL): attach a basic [P] Energy card
//! from your discard pile to 1 of your Benched [P] Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WonderPatch",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(crate::types::ct::PSYCHIC),
                scan: TargetScan::BenchEffectiveType,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                route: AttachRoute::Move,
                onto: None,
                cards: CardSel::All,
                none_shuffles: false,
             different_types: false, })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
