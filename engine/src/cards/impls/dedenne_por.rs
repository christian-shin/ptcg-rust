//! Dedenne (POR 29): Tail Generator - choose Basic [L] Energy cards from your discard pile up to the amount of Energy attached to all
//! of your opponent's Pokémon and attach them to your [L] Pokémon in any way you like. Thunder Shock - 30; flip a coin. If heads,
//! your opponent's Active Pokémon is now Paralyzed.
//!
//! Tail Generator: one Attach of 0 to as many Basic [L] Energy cards of the discard pile as the Energy attached to the opponent's
//! Pokémon provide (Provided units, id2237; the bound is read as the game checks it) onto your Pokémon that are [L] as they are now
//! (an Attach event per card, caused by the attack). Thunder Shock flips after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DedennePOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                slots: AttachSlots::ActiveBench,
                target: SlotPred::TypeIs(ct::LIGHTNING),
                bounds: Bounds { min: Num::Lit(0), max: Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedUnits) },
                ..AttachSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
