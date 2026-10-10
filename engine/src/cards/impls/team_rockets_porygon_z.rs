//! Team Rocket's Porygon-Z (DRI): Reconstitute — discard 2 cards from your
//! hand; once during your turn, draw a card. R Command — 20 damage for each
//! Team Rocket Supporter in your discard pile.
//!
//! Twinleaf: POWER_ALREADY_USED with the marker, CANNOT_USE_POWER with
//! fewer than 2 cards in hand or an empty deck, then a non-cancellable
//! ChooseCardsPrompt (exactly 2) on the hand; the callback marks, adds the
//! ability-used board effect, discards the pair and draws 1.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPorygonZ",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("RECONSTITUTE_MARKER"),
        needs: &[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Ge, Num::Lit(2)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        // Discard 2 cards from your hand; draw a card.
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
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
