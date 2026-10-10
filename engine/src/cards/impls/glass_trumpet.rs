//! Glass Trumpet (SCR): you can use this card only if you have any Tera
//! Pokémon in play. Choose up to 2 of your Benched [C] Pokémon and attach a
//! Basic Energy card from your discard pile to each of them.
//!
//! Twinleaf: the card goes hand→supporter (a no-op for an Item already
//! there) and the TrainerEffect is prevented before the checks. Fixed (phase
//! 4b): with no Benched [C] Pokémon the card throws CANNOT_PLAY_THIS_CARD
//! (the prompt below would have no legal target). AttachEnergyPrompt from the
//! discard: Bench only, min 1, max 2, different targets, blockedTo = every
//! non-[C] Pokémon (Active included).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GlassTrumpet",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA))],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(crate::types::ct::COLORLESS),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) },
                same_target: false,
                different_targets: true,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
             different_types: false, })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
