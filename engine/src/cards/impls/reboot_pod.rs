//! Reboot Pod (TEF 158, ACE SPEC): attach a Basic Energy card from your discard pile to each of your Future Pokémon.
//!
//! One Attach prompt from the discard pile: exactly as many Energy cards as there are Future Pokémon in play (or Basic Energy
//! cards in the discard pile if fewer), one per Future Pokémon, no cancel; the other Pokémon are blocked. With no Future Pokémon
//! in play, or no Basic Energy in the discard pile, it has no effect and can't be played (id925).
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
                target: SlotPred::Tag(crate::types::tag::FUTURE), scan: TargetScan::InPlay,
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
