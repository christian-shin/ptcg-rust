//! Explorer's Guidance (TEF, Ancient Supporter): look at the top 6 cards of
//! your deck and put 2 of them into your hand. Discard the other cards.
//!
//! Twinleaf: any EndTurnEffect for a player with `ancientSupporter` clears
//! it. On play: throws when a Supporter was already played; the card moves
//! to the Supporter area and the play is prevented; throws on an empty deck
//! (after that move); 6 cards go to a temporary list; the non-cancellable
//! ChooseCardsPrompt takes `min = min(2, looked at)` (fixed in phase 4b, R4:
//! it was 1 when the deck had at most 1 card left after taking the 6) and up
//! to 2; the callback sets
//! `ancientSupporter`, moves the chosen cards to the hand and the rest to the
//! discard pile.
//!
//! R7C: `ancient_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::spec::prelude::*;

const LOOKED_AT: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));

pub static SPEC: CardSpec = CardSpec {
    class: "ExplorersGuidance",
    // Look at the top 6 cards of your deck and put 2 of them into your hand. Discard the other cards.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: LOOKED_AT,
                bounds: Bounds { min: Num::Min(&Num::Lit(2), &Num::ZoneSize(LOOKED_AT)), max: Num::Lit(2) },
                into: 0,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            // Using the effect of a Supporter as the effect of an attack is not playing it from the hand.
            Step::new(Op::If(IfSpec {
                cond: Cond::Not(&Cond::TrainerViaAttack),
                yes: &[Step::new(Op::SetFlag(SetFlagSpec { who: Who::Me, flag: PlayerFlag::AncientSupporter, value: true }))],
                no: &[],
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: LOOKED_AT, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: LOOKED_AT, cards: CardSel::All, ..DiscardSpec::DEFAULT })),
        ],
    }),
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::SetFlag(SetFlagSpec { who: Who::Me, flag: PlayerFlag::AncientSupporter, value: false }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
