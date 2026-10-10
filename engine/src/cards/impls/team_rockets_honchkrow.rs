//! Team Rocket's Honchkrow (M2a): Rocket Feathers — discard any number of
//! "Team Rocket" Supporters from your hand; 60 damage for each. Hammer In —
//! 100.
//!
//! Twinleaf: the hand prompt (Trainer filter, non-Team Rocket Supporters
//! blocked, max = their count) is shown even when there are none; choosing
//! nothing sets the damage to 0.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsHonchkrow",
    attacks: &[AttackSpec {
        index: 0,
        // Discard any number of "Team Rocket" Supporters from your hand; 60 damage for each.
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::Supporter, Pred::NameContains("Team Rocket")]),
                bounds: Bounds { min: Num::Lit(0), max: Num::CardCount(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::Supporter, Pred::NameContains("Team Rocket")])) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), op: DamageOp::Set, per: 60 })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
