//! Glass Trumpet (SCR 135 / ASC 189): you can use this card only if you have any Tera Pokémon in play. Choose up to 2 of your
//! Benched [C] Pokémon and attach a Basic Energy card from your discard pile to each of them.
//!
//! "You can use this card only if" is the printed `Cond::Printed`: it still binds when the Item's effect is used by another
//! card. With no Benched [C] Pokémon or no Basic Energy in the discard pile the card has no effect and can't be played
//! (id925). One Attach from the discard pile, 1 to 2 cards, each to a different Benched Pokémon that is [C] after effects
//! (`TargetScan::EffectiveTypes`: every Pokémon in play that isn't, Active included, is a blocked destination).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GlassTrumpet",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA)))],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: SlotPred::TypeIs(crate::types::ct::COLORLESS), scan: TargetScan::InPlay,
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
