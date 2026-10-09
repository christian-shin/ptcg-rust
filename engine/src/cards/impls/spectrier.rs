//! Spectrier (ASC): Spooky Shot — 30. Phantasmal Barrage — discard all Energy
//! from this Pokémon and place 12 damage counters on 1 of your opponent's
//! Pokémon.
//!
//! Twinleaf: DISCARD_ALL_ENERGY_FROM_POKEMON (one DiscardCardsEffect with the
//! Active's CheckProvidedEnergy map), then a non-cancellable ChoosePokemonPrompt
//! whose callback reduces a PutCountersEffect (120) on the chosen Pokémon. The
//! Energy discard is queued with the attack; the Pokémon is chosen in
//! AfterAttack (after the damage; user rule 2026-10-07).
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
