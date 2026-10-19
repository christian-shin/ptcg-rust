//! Lunatone (MEG 74 / ASC 105): Lunar Cycle - once during your turn, if you have Solrock in play, you may discard a Basic
//! [F] Energy card from your hand in order to use this Ability. Draw 3 cards. You can't use more than 1 Lunar Cycle Ability
//! each turn. Power Gem - 50.
//!
//! The Energy card is the cost (`Step::cost`, APR E-30; as for N's Zoroark ex's Trade, id2115): one Discard event from the
//! hand (cause: this Ability), chosen with no cancel once the Ability is used. Then one Draw event of 3. The usage record
//! is the player's (any copy), made when the Ability is used (id127, id249), and ends with the owner's turn. Solrock may
//! be Active or Benched (by name).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lunatone",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurnShared,
        needs: &[Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("Solrock"))],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
