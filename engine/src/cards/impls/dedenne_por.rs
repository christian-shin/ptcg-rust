//! Dedenne (POR 29): Tail Generator - choose Basic [L] Energy cards from your discard pile up to the amount of Energy attached to
//! all of your opponent's Pokémon and attach them to your [L] Pokémon in any way you like. Thunder Shock - 30; flip a coin. If
//! heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! Tail Generator: 0 to as many Basic [L] Energy cards as the Energy provided to all the opponent's Pokémon (id2237: Energy
//! attached is Energy provided) and the discard pile holds, each an Attach event onto a [L] Pokémon of yours (the type it has now,
//! APR D-07), any way you like (Rosa's Encouragement's form). Thunder Shock: Litten TEF's form.
use crate::spec::prelude::*;

const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);
const BASIC_L: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "DedennePOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Nonempty(DISCARD, BASIC_L), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(ct::LIGHTNING))]),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: DISCARD,
                    predicate: BASIC_L,
                    slots: AttachSlots::ActiveBench,
                    target: SlotPred::TypeIs(ct::LIGHTNING),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedUnits), &Num::CardCount(DISCARD, BASIC_L)) },
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
