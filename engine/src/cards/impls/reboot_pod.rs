//! Reboot Pod (TEF, ACE SPEC): attach a Basic Energy card from your discard
//! pile to each of your Future Pokémon in play.
//!
//! Twinleaf: one AttachEnergyPrompt from the discard (phase 4b: exactly
//! min(Future Pokémon in play, Basic Energy in the discard pile) cards, one
//! per Pokémon; it used to be 0..energy count; no cancel, different targets,
//! non-Future Pokémon blocked); unplayable with no Future Pokémon in play
//! (phase 4b). Each transfer is a plain MOVE_CARDS discard→slot (no
//! AttachEnergyEffect) followed by a MOVE_CARDS of the card supporter→discard.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RebootPod",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: Pred::Tag(crate::types::tag::FUTURE),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy), &Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::FUTURE))), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy), &Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::FUTURE))) },
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
