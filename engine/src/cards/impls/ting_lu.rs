//! Ting-Lu (TWM 110): Ground Crasher — 30; if a Stadium is in play, 30 damage
//! to each of your opponent's Benched Pokémon, then discard that Stadium.
//! Hammer In — 110.
//!
//! One Damage event per Benched Pokémon of the opponent (no Weakness or Resistance), each with the attack as its cause,
//! then the Stadium is discarded after the damage and before the Knock Outs (the state check; rulings 1559, 1589): a
//! Pokémon that the Stadium's HP bonus or Tool lock kept alive is Knocked Out only once the Stadium is gone, and a
//! Stadium that had already left play is never replaced by a later one.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TingLu",
    attacks: &[AttackSpec {
        index: 0,
        // If a Stadium is in play, 30 damage to each of your opponent's Benched Pokémon, then discard that Stadium.
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::StadiumInPlay(Pred::Any),
            yes: &[
                Step::new(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(30), ..EachSlotSpec::DEFAULT })),
                Step::new(DISCARD_STADIUM),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
