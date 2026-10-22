//! Wo-Chien (SSP 15): Hazardous Greed - 20; if there are 3 or fewer cards in your deck, this attack also does 120 damage to 2 of
//! your opponent's Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.) Entangling Whip - 130; discard the
//! top 3 cards of your deck.
//!
//! Hazardous Greed: with 3 or fewer cards in your deck, exactly min(2, the opponent's Benched Pokémon) targets are chosen (at step D,
//! no cancel) and each takes one Damage event of 120 caused by the attack, without Weakness or Resistance (APR B-08; Ambipom PFL 79).
//! Entangling Whip: a Discard event over your own deck after the damage (Team Rocket's Larvitar DRI 94).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WoChien",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: SlotSel::Bench(Who::Opp),
                choose: Some(ChooseN {
                    chooser: Who::Me,
                    min: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)),
                    max: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)),
                }),
                what: EachWhat::Damage(DamageCalc::Put),
                amount: Num::Lit(120),
                when: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck)), CmpOp::Le, Num::Lit(3)),
                ..EachSlotSpec::DEFAULT
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(3)), ..DiscardSpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
