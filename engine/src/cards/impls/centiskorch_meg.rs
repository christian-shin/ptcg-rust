//! Centiskorch (MEG 30): Coiling Crush - 50; flip 2 coins. For each heads, discard an Energy from your opponent's Active Pokémon.
//! Heat Crawler - 140.
//!
//! After the damage 2 coins are flipped; then the attacker chooses as many Energy attached to the Defending Pokémon as there were
//! heads (as many as there are when fewer are attached, id2352) and they are discarded (a LeavePlay caused by this attack;
//! Lugia ex PRE 82's Hyper Whirlpool). This is a different card from Centiskorch SSP 28 and PBL 10.
use crate::spec::prelude::*;

const OPP_ENERGY: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(OPP_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "CentiskorchMEG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(2)),
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
