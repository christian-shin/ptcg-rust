//! Oricorio ex (M2): Excited Turbo — as often as you like during your turn,
//! if you have any [R] Mega Evolution Pokémon ex in play, attach a Basic [R]
//! Energy card from your hand to 1 of your Benched [R] Pokémon. Fire Wing — 110.
//!
//! Twinleaf: blockedTo lists every non-[R] Pokémon (Active included); the
//! prompt filter is `name: 'Fire Energy'`; each transfer is an
//! AttachEnergyEffect.
//!
//! Fixed (phase 4b, W4): the Mega ex check only looked at the cards stacked
//! in the Active Spot (now any Pokémon in play), and the card was missing its
//! `ex` tag (so a Knock Out gave 1 Prize card instead of 2).
//!
//! Fixed (phase 4b, R2): the AttachEnergyPrompt had the default min 0 / max
//! hand size, so a use could attach nothing (a free repeatable no-op) or many
//! cards; each use attaches exactly 1 Energy card (min 1, max 1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Oricorioex",
    attacks: &[],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::InPlay(
            Who::Me,
            PlayScope::All,
            Pred::All(&[Pred::Tag(crate::types::tag::POKEMON_EX_LOWER), Pred::Tag(crate::types::tag::POKEMON_SV_MEGA), Pred::PokemonType(crate::types::ct::FIRE)]),
        )],
        steps: &[Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(crate::types::ct::FIRE),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
                none_shuffles: false,
             different_types: false, }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
