//! Ninetales (MEG 20): Supernatural Shapeshifter - discard the top card of your deck, and if that card is a Supporter card,
//! use the effect of that card as the effect of this attack. Combustion - 60.
//!
//! The top card is discarded (a Discard event caused by the attack, kept in register 0); if it is a Supporter, its effect
//! is used as this attack's effect (`Op::UseTrainer` required: you can't choose not to when able, id2227; it isn't played,
//! so a Supporter already played this turn or a lock on playing one doesn't stop it, id2225, id2226; its printed conditions
//! still apply, JP FAQ Fire Energy MEE 2). A Supporter that asks to discard cards may be played with 0 (id2376).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NinetalesMEG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: Some(0) })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::Supporter),
                yes: &[Step::new(Op::UseTrainer(UseTrainerSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), kind: PlayKind::Supporter, required: true }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
