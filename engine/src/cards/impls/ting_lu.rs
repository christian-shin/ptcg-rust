//! Ting-Lu (TWM 110): Ground Crack — 30; if a Stadium is in play, 30 damage
//! to each of your opponent's Benched Pokémon, then discard that Stadium.
//! Hammer In — 110.
//!
//! Fixed (phase 4b, R7F-2; rulings 1559, 1589): Twinleaf set a card-object
//! flag in the attack and discarded the Stadium on a later BetweenTurnsEffect,
//! after the Knock Out check (a Pokémon the Stadium's HP bonus or Tool lock
//! kept alive was Knocked Out) and, when the Stadium had already left play,
//! the flag stayed set and discarded a later Stadium. The Stadium is now
//! discarded in AfterAttackEffect: after the damage, before the Knock Outs.
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
