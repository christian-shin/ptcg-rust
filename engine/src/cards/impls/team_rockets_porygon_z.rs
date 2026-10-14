//! Team Rocket's Porygon-Z (DRI): Reconstitute — you must discard 2 cards from your hand in order to use this Ability. Once
//! during your turn, you may draw a card. R Command — 20×; this attack does 20 damage for each Supporter card that has "Team
//! Rocket" in its name in your discard pile.
//!
//! The discard is the Ability's cost, `Step::cost` (APR E-30): it needs 2 cards in the hand and a card in the deck (a card
//! that can't draw has no effect: id2231). The Draw is one Draw event of the Ability's effect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPorygonZ",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        // Discard 2 cards from your hand; draw a card.
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, purpose: Some(PromptPurpose::DiscardCard), ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    attacks: &[AttackSpec {
        index: 0,
        // R Command: 20 damage for each Team Rocket Supporter in your discard pile.
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Supporter, Pred::NameContains("Team Rocket")])), &Num::Lit(20))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
