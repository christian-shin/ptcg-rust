//! N's Darmanitan (JTG): Backdraft — 30 damage for each Basic Energy card in
//! your opponent's discard pile. Darman-i-cannon — 90; discard all Energy
//! from this Pokémon; also 90 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: Backdraft assigns the damage. Darman-i-cannon discards every
//! card of the Active's CheckProvidedEnergy map with one DiscardCardsEffect,
//! then (when the opponent has a Benched Pokémon; phase 4b fix, it used to
//! open the prompt on an empty Bench and get stuck) opens a non-cancellable
//! ChoosePokemonPrompt over the opponent's Bench, and puts 90 with a plain
//! PutDamageEffect on the chosen Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsDarmanitan",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Opp, Zone::Discard), Pred::BasicEnergy), &Num::Lit(30)), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::AllProvided })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(90), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
