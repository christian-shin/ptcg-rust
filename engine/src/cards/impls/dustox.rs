//! Dustox (ASC 15): Boisterous Wind - once during your turn, you may use this Ability. Flip a coin. If heads, put an Energy attached
//! to your opponent's Active Pokémon into their hand. Twilight Poison - 100; your opponent's Active Pokémon is now Asleep and
//! Poisoned.
//!
//! Boisterous Wind is once per turn for each copy: the coin is flipped whatever is attached (the flip is part of the effect, APR
//! D-01); on heads the player using it chooses 1 Energy card attached to the Defending Pokémon (the only one when it holds one) and it
//! leaves play for its owner's hand (a LeavePlay caused by the Ability; Volcanion MEG 25's form on the opponent's Pokémon).
use crate::spec::prelude::*;

const OPP_ENERGY: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(OPP_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "Dustox",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::Cards {
                    min: Num::Min(&Num::Lit(1), &Num::CardCount(OPP_ENERGY, Pred::Energy)),
                    max: Num::Min(&Num::Lit(1), &Num::CardCount(OPP_ENERGY, Pred::Energy)),
                    kind: EnergyKind::Any,
                    cancel: false,
                    energies_only: true,
                },
                to: EnergyDest::Hand,
                ..DiscardEnergySpec::DEFAULT
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep, SpecialCondition::Poisoned]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
