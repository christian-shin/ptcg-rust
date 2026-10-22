//! Lugia ex (PRE 82): Hyper Whirlpool - 140; flip a coin until you get tails. For each heads, discard an Energy from your opponent's
//! Active Pokémon.
//!
//! After the damage the coins are flipped until tails; then the attacker chooses as many Energy attached to the Defending Pokémon
//! as there were heads (as many as there are when fewer are attached, id2352) and they are discarded (a LeavePlay caused by this
//! attack).
use crate::spec::prelude::*;

const OPP_ENERGY: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(OPP_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "LugiaEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::UntilTails,
            then: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::Cards {
                    min: Num::Min(&Num::Heads, &Num::CardCount(OPP_ENERGY, Pred::Energy)),
                    max: Num::Min(&Num::Heads, &Num::CardCount(OPP_ENERGY, Pred::Energy)),
                    kind: EnergyKind::Any,
                    cancel: false,
                    energies_only: true,
                },
                ..DiscardEnergySpec::DEFAULT
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
