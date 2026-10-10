//! Spectrier (ASC): Spooky Shot — 30. Phantasmal Barrage — discard all Energy
//! from this Pokémon and place 12 damage counters on 1 of your opponent's
//! Pokémon.
//!
//! The Energy is discarded, then the Pokémon is chosen (non-cancellable, at step D of the attack). The counters are a
//! PlaceCounters event with the attack as its cause (APR C-07: not damage): a Pokémon protected from the effects of
//! attacks can be chosen and gets none, and Battle Cage refuses them on a Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Spectrier",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::PlaceCounters(PlaceCountersSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                counters: Num::Lit(12)
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
