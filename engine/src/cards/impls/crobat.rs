//! Crobat (CRI 51): Nighttime Maneuvers - once during your turn, if this Pokémon is in the Active Spot, you may use this Ability.
//! Search your deck for a card. Shuffle your deck, then put that card on top of it. Poison Sound Wave - 80; your opponent's Active
//! Pokémon is now Confused and Poisoned.
//!
//! An Ability (once per turn for each copy) printed for the Active Spot only (`Cond::Printed`): one card of the deck is chosen (the deck is
//! hidden: any card, none when it is empty), set aside while the deck is shuffled (APR E-19), then put on top by one PutIntoDeck
//! event of the Ability (Ciphermaniac's Codebreaking with one card). Poison Sound Wave puts both conditions on the Defending
//! Pokémon after the damage.
//!
//! It needs at least 2 cards in the deck (APR B-01 / A-02 step 1, id925: an Ability that can't change the game can't be used):
//! with 1 card, choosing it, shuffling and putting it back on top changes nothing, and with 0 the search can't be made (id840).
//! (The rules' walk does not see that a Pick of the whole deck followed by a put back on top is a no-op, so the Ability states it.)
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Crobat",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Printed(&Cond::IsActive(SlotExpr::This)), Cond::Cmp(Num::ZoneSize(DECK), CmpOp::Gt, Num::Lit(1))],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: DECK, bounds: Bounds { min: Num::Min(&Num::Lit(1), &Num::ZoneSize(DECK)), max: Num::Min(&Num::Lit(1), &Num::ZoneSize(DECK)) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::Look(LookSpec { from: DECK, cards: CardSel::Chosen(0), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(1)), cards: CardSel::All, position: DeckPosition::Top, order: DeckOrder::AsIs, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused, SpecialCondition::Poisoned]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
