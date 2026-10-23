//! Delphox (CRI 13): Flaring Magic - once during your turn, you may discard a Basic [R] Energy card from your hand in order to use
//! this Ability. Draw cards until you have 7 cards in your hand. Energized Storm - 30x; this attack does 30 damage for each Energy
//! attached to all Pokémon.
//!
//! The Energy card is the cost (`Step::cost`, APR E-30; Lunatone MEG 74): one Discard event from the hand (cause: this Ability), then
//! one Draw event up to 7 cards. With 8 or more cards in hand the cost can't be paid usefully and the Ability can't be used (id2343:
//! after the discard nothing would be drawn). Energized Storm counts the Energy provided on both players' Pokémon, in units,
//! before the damage (Leafeon ex's count, over both sides).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Delphox",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(7)) })),
        ],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(
            &Num::Add(&Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::ProvidedUnits), &Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedUnits)),
            &Num::Lit(30),
        )))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
