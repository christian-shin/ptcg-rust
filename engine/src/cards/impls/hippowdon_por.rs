//! Hippowdon (POR 40): Twister Spewing - 80; if you played Tarragon from your hand during this turn, discard the top 3 cards of your
//! opponent's deck. Heavy Impact - 130. This is a different card from Hippowdon DRI 106.
//!
//! After the damage, when a Tarragon was played from the hand this turn (the played-this-turn record, APR E-26) the top 3 cards of the
//! opponent's deck (all when fewer) are discarded (a Discard caused by the attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HippowdonPOR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::PlayedThisTurn(Who::Me, Pred::Name("Tarragon")),
            yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(3)), ..DiscardSpec::DEFAULT }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
