//! Wo-Chien (SSP 15): Hazardous Greed - 20; if there are 3 or fewer cards in your deck, this attack also does 120 damage to 2 of your
//! opponent's Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.) Entangling Whip - 130; discard the top 3
//! cards of your deck.
//!
//! The deck is read after the printed damage; with the condition true, exactly min(2, the opponent's Benched Pokémon) targets, each a
//! Damage event of 120 (APR B-08; Steven's Skarmory's Sonic Double). Entangling Whip is one Discard of the top 3 cards of your deck
//! (Haxorus SFA's Dragon Pulse).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "WoChien",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: SlotSel::Bench(Who::Opp),
                choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)), max: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)) }),
                what: EachWhat::Damage(DamageCalc::Auto),
                amount: Num::Lit(120),
                when: Cond::Cmp(Num::ZoneSize(DECK), CmpOp::Le, Num::Lit(3)),
                ..EachSlotSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: DECK, cards: CardSel::Top(Num::Lit(3)), ..DiscardSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
